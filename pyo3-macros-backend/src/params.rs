use crate::{
    method::{FnArg, FnSpec},
    pyfunction::FunctionSignature,
    quotes::some_wrap,
};
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::Result;

/// Return true if the argument list is simply (*args, **kwds).
pub fn is_forwarded_args(signature: &FunctionSignature<'_>) -> bool {
    matches!(
        signature.arguments.as_slice(),
        [
            FnArg {
                is_varargs: true,
                ..
            },
            FnArg {
                is_kwargs: true,
                ..
            },
        ]
    )
}

pub fn impl_arg_params(
    spec: &FnSpec<'_>,
    self_: Option<&syn::Type>,
    fastcall: bool,
    holders: &mut Vec<TokenStream>,
) -> Result<(TokenStream, Vec<TokenStream>)> {
    let args_array = syn::Ident::new("output", Span::call_site());

    if !fastcall && is_forwarded_args(&spec.signature) {
        // In the varargs convention, we can just pass though if the signature
        // is (*args, **kwds).
        let arg_convert = spec
            .signature
            .arguments
            .iter()
            .map(|arg| impl_arg_param(arg, &mut 0, &args_array, holders))
            .collect::<Result<_>>()?;
        return Ok((
            quote! {
                let _args = py.from_borrowed_ptr::<_pyo3::types::PyTuple>(_args);
                let _kwargs: ::std::option::Option<&_pyo3::types::PyDict> = py.from_borrowed_ptr_or_opt(_kwargs);
            },
            arg_convert,
        ));
    };

    let positional_parameter_names = &spec.signature.python_signature.positional_parameters;
    let positional_only_parameters = &spec.signature.python_signature.positional_only_parameters;
    let required_positional_parameters = &spec
        .signature
        .python_signature
        .required_positional_parameters;
    let keyword_only_parameters = spec
        .signature
        .python_signature
        .keyword_only_parameters
        .iter()
        .map(|(name, required)| {
            quote! {
                _pyo3::impl_::extract_argument::KeywordOnlyParameterDescription {
                    name: #name,
                    required: #required,
                }
            }
        });

    let num_params = positional_parameter_names.len() + keyword_only_parameters.len();

    let mut option_pos = 0;
    let param_conversion = spec
        .signature
        .arguments
        .iter()
        .map(|arg| impl_arg_param(arg, &mut option_pos, &args_array, holders))
        .collect::<Result<_>>()?;

    let args_handler = if spec.signature.python_signature.varargs.is_some() {
        quote! { _pyo3::impl_::extract_argument::TupleVarargs }
    } else {
        quote! { _pyo3::impl_::extract_argument::NoVarargs }
    };
    let kwargs_handler = if spec.signature.python_signature.kwargs.is_some() {
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
                py,
                _args,
                _nargs,
                _kwnames,
                &mut #args_array
            )?
        }
    } else {
        quote! {
            DESCRIPTION.extract_arguments_tuple_dict::<#args_handler, #kwargs_handler>(
                py,
                _args,
                _kwargs,
                &mut #args_array
            )?
        }
    };

    // create array of arguments, and then parse
    Ok((
        quote! {
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
        },
        param_conversion,
    ))
}

/// Re option_pos: The option slice doesn't contain the py: Python argument, so the argument
/// index and the index in option diverge when using py: Python
fn impl_arg_param(
    arg: &FnArg<'_>,
    option_pos: &mut usize,
    args_array: &syn::Ident,
    holders: &mut Vec<TokenStream>,
) -> Result<TokenStream> {
    // Use this macro inside this function, to ensure that all code generated here is associated
    // with the function argument
    macro_rules! quote_arg_span {
        ($($tokens:tt)*) => { quote_spanned!(arg.ty.span() => $($tokens)*) }
    }

    if arg.py {
        return Ok(quote! { py });
    }

    if arg.is_cancel_handle {
        return Ok(quote! { __cancel_handle });
    }

    let name = arg.name;
    let name_str = name.to_string();

    let mut push_holder = || {
        let holder = syn::Ident::new(&format!("holder_{}", holders.len()), arg.ty.span());
        holders.push(quote_arg_span! {
            #[allow(clippy::let_unit_value)]
            let mut #holder = _pyo3::impl_::extract_argument::FunctionArgumentHolder::INIT;
        });
        holder
    };

    if arg.is_varargs {
        ensure_spanned!(
            arg.optional.is_none(),
            arg.name.span() => "args cannot be optional"
        );
        let holder = push_holder();
        return Ok(quote_arg_span! {
            _pyo3::impl_::extract_argument::extract_argument(
                _args,
                &mut #holder,
                #name_str
            )?
        });
    } else if arg.is_kwargs {
        ensure_spanned!(
            arg.optional.is_some(),
            arg.name.span() => "kwargs must be Option<_>"
        );
        let holder = push_holder();
        return Ok(quote_arg_span! {
            _pyo3::impl_::extract_argument::extract_optional_argument(
                _kwargs.map(::std::convert::AsRef::as_ref),
                &mut #holder,
                #name_str,
                || ::std::option::Option::None
            )?
        });
    }

    let arg_value = quote_arg_span!(#args_array[#option_pos]);
    *option_pos += 1;

    let mut default = arg.default.as_ref().map(|expr| quote!(#expr));

    // Option<T> arguments have special treatment: the default should be specified _without_ the
    // Some() wrapper. Maybe this should be changed in future?!
    if arg.optional.is_some() {
        default = Some(default.map_or_else(|| quote!(::std::option::Option::None), some_wrap));
    }

    let tokens = if let Some(expr_path) = arg.attrs.from_py_with.as_ref().map(|attr| &attr.value) {
        if let Some(default) = default {
            quote_arg_span! {
                #[allow(clippy::redundant_closure)]
                _pyo3::impl_::extract_argument::from_py_with_with_default(
                    #arg_value,
                    #name_str,
                    #expr_path,
                    || #default
                )?
            }
        } else {
            quote_arg_span! {
                _pyo3::impl_::extract_argument::from_py_with(
                    _pyo3::impl_::extract_argument::unwrap_required_argument(#arg_value),
                    #name_str,
                    #expr_path,
                )?
            }
        }
    } else if arg.optional.is_some() {
        let holder = push_holder();
        quote_arg_span! {
            #[allow(clippy::redundant_closure)]
            _pyo3::impl_::extract_argument::extract_optional_argument(
                #arg_value,
                &mut #holder,
                #name_str,
                || #default
            )?
        }
    } else if let Some(default) = default {
        let holder = push_holder();
        quote_arg_span! {
            #[allow(clippy::redundant_closure)]
            _pyo3::impl_::extract_argument::extract_argument_with_default(
                #arg_value,
                &mut #holder,
                #name_str,
                || #default
            )?
        }
    } else {
        let holder = push_holder();
        quote_arg_span! {
            _pyo3::impl_::extract_argument::extract_argument(
                _pyo3::impl_::extract_argument::unwrap_required_argument(#arg_value),
                &mut #holder,
                #name_str
            )?
        }
    };
    Ok(tokens)
}
