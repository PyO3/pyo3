// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::{
    attributes::FromPyWithAttribute,
    method::{FnArg, FnSpec},
    pyfunction::Argument,
    utils::{remove_lifetime, unwrap_ty_group},
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
    py: &syn::Ident,
    fastcall: bool,
) -> Result<TokenStream> {
    if spec.args.is_empty() {
        return Ok(TokenStream::new());
    }

    let args_array = syn::Ident::new("output", Span::call_site());

    if !fastcall && is_forwarded_args(&spec.args, &spec.attrs) {
        // In the varargs convention, we can just pass though if the signature
        // is (*args, **kwds).
        let mut arg_convert = vec![];
        for (i, arg) in spec.args.iter().enumerate() {
            arg_convert.push(impl_arg_param(arg, spec, i, &mut 0, py, &args_array)?);
        }
        return Ok(quote! {
            let _args = #py.from_borrowed_ptr::<_pyo3::types::PyTuple>(_args);
            let _kwargs: ::std::option::Option<&_pyo3::types::PyDict> = #py.from_borrowed_ptr_or_opt(_kwargs);
            #(#arg_convert)*
        });
    };

    let mut positional_parameter_names = Vec::new();
    let mut positional_only_parameters = 0usize;
    let mut required_positional_parameters = 0usize;
    let mut keyword_only_parameters = Vec::new();

    for arg in &spec.args {
        if arg.py || is_args(&spec.attrs, arg.name) || is_kwargs(&spec.attrs, arg.name) {
            continue;
        }
        let name = arg.name.unraw().to_string();
        let posonly = spec.is_pos_only(arg.name);
        let kwonly = spec.is_kw_only(arg.name);
        let required = !(arg.optional.is_some() || spec.default_value(arg.name).is_some());

        if kwonly {
            keyword_only_parameters.push(quote! {
                _pyo3::impl_::extract_argument::KeywordOnlyParameterDescription {
                    name: #name,
                    required: #required,
                }
            });
        } else {
            positional_parameter_names.push(name);

            if required {
                required_positional_parameters = positional_parameter_names.len();
            }
            if posonly {
                positional_only_parameters += 1;
            }
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
            &mut option_pos,
            py,
            &args_array,
        )?);
    }

    let (accept_args, accept_kwargs) = accept_args_kwargs(&spec.attrs);
    let args_handler = if accept_args {
        quote! { _pyo3::impl_::extract_argument::TupleVarargs }
    } else {
        quote! { _pyo3::impl_::extract_argument::NoVarargs }
    };
    let kwargs_handler = if accept_kwargs {
        quote! { _pyo3::impl_::extract_argument::DictVarkeywords }
    } else {
        quote! { _pyo3::impl_::extract_argument::NoVarkeywords }
    };

    let cls_name = if let Some(cls) = self_ {
        quote! { ::std::option::Option::Some(<#cls as _pyo3::type_object::PyTypeInfo>::NAME) }
    } else {
        quote! { ::std::option::Option::None }
    };
    let python_name = &spec.python_name;

    let extract_expression = if fastcall {
        quote! {
            DESCRIPTION.extract_arguments_fastcall::<#args_handler, #kwargs_handler>(
                #py,
                _args,
                _nargs,
                _kwnames,
                &mut #args_array
            )?
        }
    } else {
        quote! {
            DESCRIPTION.extract_arguments_tuple_dict::<#args_handler, #kwargs_handler>(
                #py,
                _args,
                _kwargs,
                &mut #args_array
            )?
        }
    };

    // create array of arguments, and then parse
    Ok(quote! {
            const DESCRIPTION: _pyo3::impl_::extract_argument::FunctionDescription = _pyo3::impl_::extract_argument::FunctionDescription {
                cls_name: #cls_name,
                func_name: stringify!(#python_name),
                positional_parameter_names: &[#(#positional_parameter_names),*],
                positional_only_parameters: #positional_only_parameters,
                required_positional_parameters: #required_positional_parameters,
                keyword_only_parameters: &[#(#keyword_only_parameters),*],
            };

            let mut #args_array = [::std::option::Option::None; #num_params];
            let (_args, _kwargs) = #extract_expression;

            #(#param_conversion)*
    })
}

/// Re option_pos: The option slice doesn't contain the py: Python argument, so the argument
/// index and the index in option diverge when using py: Python
fn impl_arg_param(
    arg: &FnArg<'_>,
    spec: &FnSpec<'_>,
    idx: usize,
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
    let name_str = name.to_string();

    if is_args(&spec.attrs, name) {
        ensure_spanned!(
            arg.optional.is_none(),
            arg.name.span() => "args cannot be optional"
        );
        return Ok(quote_arg_span! {
            let #arg_name = _pyo3::impl_::extract_argument::extract_argument(_args, #name_str)?;
        });
    } else if is_kwargs(&spec.attrs, name) {
        ensure_spanned!(
            arg.optional.is_some(),
            arg.name.span() => "kwargs must be Option<_>"
        );
        return Ok(quote_arg_span! {
            let #arg_name = _pyo3::impl_::extract_argument::extract_optional_argument(_kwargs.map(|kwargs| kwargs.as_ref()), #name_str)?;
        });
    }

    let arg_value = quote_arg_span!(#args_array[#option_pos]);
    *option_pos += 1;

    let arg_value_or_default = if let Some(FromPyWithAttribute {
        value: expr_path, ..
    }) = &arg.attrs.from_py_with
    {
        match (spec.default_value(name), arg.optional.is_some()) {
            (Some(default), true) if default.to_string() != "None" => {
                quote_arg_span! {
                    _pyo3::impl_::extract_argument::from_py_with_with_default(#arg_value, #name_str, #expr_path, || Some(#default))
                }
            }
            (Some(default), _) => {
                quote_arg_span! {
                    _pyo3::impl_::extract_argument::from_py_with_with_default(#arg_value, #name_str, #expr_path, || #default)
                }
            }
            (None, true) => {
                quote_arg_span! {
                    _pyo3::impl_::extract_argument::from_py_with_with_default(#arg_value, #name_str, #expr_path, || Some(None))
                }
            }
            (None, false) => {
                quote_arg_span! {
                    _pyo3::impl_::extract_argument::from_py_with(
                        _pyo3::impl_::extract_argument::unwrap_required_argument(#arg_value),
                        #name_str,
                        #expr_path,
                    )
                }
            }
        }
    } else {
        match (spec.default_value(name), arg.optional.is_some()) {
            (Some(default), true) if default.to_string() != "None" => {
                quote_arg_span! {
                    _pyo3::impl_::extract_argument::extract_argument_with_default(#arg_value, #name_str, || Some(#default))
                }
            }
            (Some(default), _) => {
                quote_arg_span! {
                    _pyo3::impl_::extract_argument::extract_argument_with_default(#arg_value, #name_str, || #default)
                }
            }
            (None, true) => {
                quote_arg_span! {
                    _pyo3::impl_::extract_argument::extract_optional_argument(#arg_value, #name_str)
                }
            }
            (None, false) => {
                quote_arg_span! {
                    _pyo3::impl_::extract_argument::extract_argument(
                        _pyo3::impl_::extract_argument::unwrap_required_argument(#arg_value),
                        #name_str
                    )
                }
            }
        }
    };

    return if let syn::Type::Reference(tref) = unwrap_ty_group(arg.optional.unwrap_or(ty)) {
        let tref = remove_lifetime(tref);
        let mut_ = tref.mutability;
        let (target_ty, borrow_tmp) = if arg.optional.is_some() {
            // Get Option<&T> from Option<PyRef<T>>
            (
                quote_arg_span! { ::std::option::Option<<#tref as _pyo3::derive_utils::ExtractExt<'_>>::Target> },
                if mut_.is_some() {
                    quote_arg_span! { _tmp.as_deref_mut() }
                } else {
                    quote_arg_span! { _tmp.as_deref() }
                },
            )
        } else {
            // Get &T from PyRef<T>
            (
                quote_arg_span! { <#tref as _pyo3::derive_utils::ExtractExt<'_>>::Target },
                quote_arg_span! { &#mut_ *_tmp },
            )
        };

        Ok(quote_arg_span! {
            let #mut_ _tmp: #target_ty = #arg_value_or_default?;
            #[allow(clippy::needless_option_as_deref, clippy::borrow_deref_ref)]
            let #arg_name = #borrow_tmp;
        })
    } else if cfg!(feature = "pyproto") {
        // arg.ty is kinda weird
        Ok(quote_arg_span! {
            let #arg_name = #arg_value_or_default?;
        })
    } else {
        let ty = arg.ty;
        Ok(quote_arg_span! {
            let #arg_name: #ty = #arg_value_or_default?;
        })
    };
}
