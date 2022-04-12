// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::{
    attributes::FromPyWithAttribute,
    method::{FnArg, FnSpec},
    pyfunction::Argument,
    utils::{remove_lifetime, replace_self, unwrap_ty_group},
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::ext::IdentExt;
use syn::spanned::Spanned;
use syn::Result;

/// Determine if the function gets passed a *args tuple or **kwargs dict.
pub fn accept_args_kwargs(attrs: &[Argument]) -> (bool, bool) {
    let (mut accept_args, mut accept_kwargs) = (false, false);

    for s in attrs {
        match s {
            Argument::VarArgs(_) => accept_args = true,
            Argument::KeywordArgs(_) => accept_kwargs = true,
            _ => continue,
        }
    }

    (accept_args, accept_kwargs)
}

/// Return true if the argument list is simply (*args, **kwds).
pub fn is_forwarded_args(args: &[FnArg<'_>], attrs: &[Argument]) -> bool {
    args.len() == 2 && is_args(attrs, args[0].name) && is_kwargs(attrs, args[1].name)
}

fn is_args(attrs: &[Argument], name: &syn::Ident) -> bool {
    for s in attrs.iter() {
        if let Argument::VarArgs(path) = s {
            return path.is_ident(name);
        }
    }
    false
}

fn is_kwargs(attrs: &[Argument], name: &syn::Ident) -> bool {
    for s in attrs.iter() {
        if let Argument::KeywordArgs(path) = s {
            return path.is_ident(name);
        }
    }
    false
}

pub fn impl_arg_params(
    spec: &FnSpec<'_>,
    self_: Option<&syn::Type>,
    body: TokenStream,
    py: &syn::Ident,
    fastcall: bool,
) -> Result<TokenStream> {
    if spec.args.is_empty() {
        return Ok(body);
    }

    let args_array = syn::Ident::new("output", Span::call_site());

    if !fastcall && is_forwarded_args(&spec.args, &spec.attrs) {
        // In the varargs convention, we can just pass though if the signature
        // is (*args, **kwds).
        let mut arg_convert = vec![];
        for (i, arg) in spec.args.iter().enumerate() {
            arg_convert.push(impl_arg_param(arg, spec, i, None, &mut 0, py, &args_array)?);
        }
        return Ok(quote! {{
            let _args = Some(_args);
            #(#arg_convert)*
            #body
        }});
    };

    let mut positional_parameter_names = Vec::new();
    let mut positional_only_parameters = 0usize;
    let mut required_positional_parameters = 0usize;
    let mut keyword_only_parameters = Vec::new();

    for arg in spec.args.iter() {
        if arg.py || is_args(&spec.attrs, arg.name) || is_kwargs(&spec.attrs, arg.name) {
            continue;
        }
        let name = arg.name.unraw().to_string();
        let posonly = spec.is_pos_only(arg.name);
        let kwonly = spec.is_kw_only(arg.name);
        let required = !(arg.optional.is_some() || spec.default_value(arg.name).is_some());

        if kwonly {
            keyword_only_parameters.push(quote! {
                ::pyo3::derive_utils::KeywordOnlyParameterDescription {
                    name: #name,
                    required: #required,
                }
            });
        } else {
            if required {
                required_positional_parameters += 1;
            }
            if posonly {
                positional_only_parameters += 1;
            }
            positional_parameter_names.push(name);
        }
    }

    let num_params = positional_parameter_names.len() + keyword_only_parameters.len();

    let mut param_conversion = Vec::new();
    let mut option_pos = 0;
    for (idx, arg) in spec.args.iter().enumerate() {
        param_conversion.push(impl_arg_param(
            arg,
            spec,
            idx,
            self_,
            &mut option_pos,
            py,
            &args_array,
        )?);
    }

    let (accept_args, accept_kwargs) = accept_args_kwargs(&spec.attrs);

    let cls_name = if let Some(cls) = self_ {
        quote! { ::std::option::Option::Some(<#cls as ::pyo3::type_object::PyTypeInfo>::NAME) }
    } else {
        quote! { ::std::option::Option::None }
    };
    let python_name = &spec.python_name;

    let (args_to_extract, kwargs_to_extract) = if fastcall {
        // _args is a &[&PyAny], _kwnames is a Option<&PyTuple> containing the
        // keyword names of the keyword args in _kwargs
        (
            // need copied() for &&PyAny -> &PyAny
            quote! { ::std::iter::Iterator::copied(_args.iter()) },
            quote! { _kwnames.map(|kwnames| {
                use ::std::iter::Iterator;
                kwnames.as_slice().iter().copied().zip(_kwargs.iter().copied())
            }) },
        )
    } else {
        // _args is a &PyTuple, _kwargs is an Option<&PyDict>
        (
            quote! { _args.iter() },
            quote! { _kwargs.map(|dict| dict.iter()) },
        )
    };

    // create array of arguments, and then parse
    Ok(quote! {{
            const DESCRIPTION: ::pyo3::derive_utils::FunctionDescription = ::pyo3::derive_utils::FunctionDescription {
                cls_name: #cls_name,
                func_name: stringify!(#python_name),
                positional_parameter_names: &[#(#positional_parameter_names),*],
                positional_only_parameters: #positional_only_parameters,
                required_positional_parameters: #required_positional_parameters,
                keyword_only_parameters: &[#(#keyword_only_parameters),*],
                accept_varargs: #accept_args,
                accept_varkeywords: #accept_kwargs,
            };

            let mut #args_array = [::std::option::Option::None; #num_params];
            let (_args, _kwargs) = DESCRIPTION.extract_arguments(
                #py,
                #args_to_extract,
                #kwargs_to_extract,
                &mut #args_array
            )?;

            #(#param_conversion)*

            #body
    }})
}

/// Re option_pos: The option slice doesn't contain the py: Python argument, so the argument
/// index and the index in option diverge when using py: Python
fn impl_arg_param(
    arg: &FnArg<'_>,
    spec: &FnSpec<'_>,
    idx: usize,
    self_: Option<&syn::Type>,
    option_pos: &mut usize,
    py: &syn::Ident,
    args_array: &syn::Ident,
) -> Result<TokenStream> {
    // Use this macro inside this function, to ensure that all code generated here is associated
    // with the function argument
    macro_rules! quote_arg_span {
        ($($tokens:tt)*) => { quote_spanned!(arg.ty.span() => $($tokens)*) }
    }

    let arg_name = syn::Ident::new(&format!("arg{}", idx), Span::call_site());

    if arg.py {
        return Ok(quote_arg_span! { let #arg_name = #py; });
    }

    let ty = arg.ty;
    let name = arg.name;
    let transform_error = quote! {
        |e| ::pyo3::derive_utils::argument_extraction_error(#py, stringify!(#name), e)
    };

    if is_args(&spec.attrs, name) {
        ensure_spanned!(
            arg.optional.is_none(),
            arg.name.span() => "args cannot be optional"
        );
        return Ok(quote_arg_span! {
            let #arg_name = _args.unwrap().extract().map_err(#transform_error)?;
        });
    } else if is_kwargs(&spec.attrs, name) {
        ensure_spanned!(
            arg.optional.is_some(),
            arg.name.span() => "kwargs must be Option<_>"
        );
        return Ok(quote_arg_span! {
            let #arg_name = _kwargs.map(|kwargs| kwargs.extract())
                .transpose()
                .map_err(#transform_error)?;
        });
    }

    let arg_value = quote_arg_span!(#args_array[#option_pos]);
    *option_pos += 1;

    let extract = if let Some(FromPyWithAttribute(expr_path)) = &arg.attrs.from_py_with {
        quote_arg_span! { #expr_path(_obj).map_err(#transform_error) }
    } else {
        quote_arg_span! { _obj.extract().map_err(#transform_error) }
    };

    let arg_value_or_default = match (spec.default_value(name), arg.optional.is_some()) {
        (Some(default), true) if default.to_string() != "None" => {
            quote_arg_span! {
                #arg_value.map_or_else(|| ::std::result::Result::Ok(::std::option::Option::Some(#default)),
                                       |_obj| #extract)?
            }
        }
        (Some(default), _) => {
            quote_arg_span! {
                #arg_value.map_or_else(|| ::std::result::Result::Ok(#default), |_obj| #extract)?
            }
        }
        (None, true) => {
            quote_arg_span! {
                #arg_value.map_or(::std::result::Result::Ok(::std::option::Option::None),
                                  |_obj| #extract)?
            }
        }
        (None, false) => {
            quote_arg_span! {
                {
                    let _obj = #arg_value.expect("Failed to extract required method argument");
                    #extract?
                }
            }
        }
    };

    if let syn::Type::Reference(tref) = unwrap_ty_group(arg.optional.unwrap_or(ty)) {
        let mut tref = remove_lifetime(tref);
        if let Some(cls) = self_ {
            replace_self(&mut tref.elem, cls);
        }
        let mut_ = tref.mutability;
        let (target_ty, borrow_tmp) = if arg.optional.is_some() {
            // Get Option<&T> from Option<PyRef<T>>
            (
                quote_arg_span! { ::std::option::Option<<#tref as ::pyo3::derive_utils::ExtractExt<'_>>::Target> },
                if mut_.is_some() {
                    quote_arg_span! { _tmp.as_deref_mut() }
                } else {
                    quote_arg_span! { _tmp.as_deref() }
                },
            )
        } else {
            // Get &T from PyRef<T>
            (
                quote_arg_span! { <#tref as ::pyo3::derive_utils::ExtractExt<'_>>::Target },
                quote_arg_span! { &#mut_ *_tmp },
            )
        };

        Ok(quote_arg_span! {
            let #mut_ _tmp: #target_ty = #arg_value_or_default;
            let #arg_name = #borrow_tmp;
        })
    } else {
        Ok(quote_arg_span! {
            let #arg_name = #arg_value_or_default;
        })
    }
}
