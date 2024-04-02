use crate::utils::Ctx;
use crate::{
    method::{FnArg, FnSpec, RegularArg},
    pyfunction::FunctionSignature,
    quotes::some_wrap,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::spanned::Spanned;

pub struct Holders {
    holders: Vec<syn::Ident>,
    gil_refs_checkers: Vec<syn::Ident>,
}

impl Holders {
    pub fn new() -> Self {
        Holders {
            holders: Vec::new(),
            gil_refs_checkers: Vec::new(),
        }
    }

    pub fn push_holder(&mut self, span: Span) -> syn::Ident {
        let holder = syn::Ident::new(&format!("holder_{}", self.holders.len()), span);
        self.holders.push(holder.clone());
        holder
    }

    pub fn push_gil_refs_checker(&mut self, span: Span) -> syn::Ident {
        let gil_refs_checker = syn::Ident::new(
            &format!("gil_refs_checker_{}", self.gil_refs_checkers.len()),
            span,
        );
        self.gil_refs_checkers.push(gil_refs_checker.clone());
        gil_refs_checker
    }

    pub fn init_holders(&self, ctx: &Ctx) -> TokenStream {
        let Ctx { pyo3_path } = ctx;
        let holders = &self.holders;
        let gil_refs_checkers = &self.gil_refs_checkers;
        quote! {
            #[allow(clippy::let_unit_value)]
            #(let mut #holders = #pyo3_path::impl_::extract_argument::FunctionArgumentHolder::INIT;)*
            #(let #gil_refs_checkers = #pyo3_path::impl_::deprecations::GilRefs::new();)*
        }
    }

    pub fn check_gil_refs(&self) -> TokenStream {
        self.gil_refs_checkers
            .iter()
            .map(|e| quote_spanned! { e.span() => #e.function_arg(); })
            .collect()
    }
}

/// Return true if the argument list is simply (*args, **kwds).
pub fn is_forwarded_args(signature: &FunctionSignature<'_>) -> bool {
    matches!(
        signature.arguments.as_slice(),
        [FnArg::VarArgs(..), FnArg::KwArgs(..),]
    )
}

pub(crate) fn check_arg_for_gil_refs(
    tokens: TokenStream,
    gil_refs_checker: syn::Ident,
    ctx: &Ctx,
) -> TokenStream {
    let Ctx { pyo3_path } = ctx;
    quote! {
        #pyo3_path::impl_::deprecations::inspect_type(#tokens, &#gil_refs_checker)
    }
}

pub fn impl_arg_params(
    spec: &FnSpec<'_>,
    self_: Option<&syn::Type>,
    fastcall: bool,
    holders: &mut Holders,
    ctx: &Ctx,
) -> (TokenStream, Vec<TokenStream>) {
    let args_array = syn::Ident::new("output", Span::call_site());
    let Ctx { pyo3_path } = ctx;

    let from_py_with = spec
        .signature
        .arguments
        .iter()
        .enumerate()
        .filter_map(|(i, arg)| {
            let from_py_with = &arg.from_py_with().as_ref()?.value;
            let from_py_with_holder = format_ident!("from_py_with_{}", i);
            Some(quote_spanned! { from_py_with.span() =>
                let e = #pyo3_path::impl_::deprecations::GilRefs::new();
                let #from_py_with_holder = #pyo3_path::impl_::deprecations::inspect_fn(#from_py_with, &e);
                e.from_py_with_arg();
            })
        })
        .collect::<TokenStream>();

    if !fastcall && is_forwarded_args(&spec.signature) {
        // In the varargs convention, we can just pass though if the signature
        // is (*args, **kwds).
        let arg_convert = spec
            .signature
            .arguments
            .iter()
            .enumerate()
            .map(|(i, arg)| impl_arg_param(arg, i, &mut 0, holders, ctx))
            .collect();
        return (
            quote! {
                let _args = #pyo3_path::impl_::pymethods::BoundRef::ref_from_ptr(py, &_args);
                let _kwargs = #pyo3_path::impl_::pymethods::BoundRef::ref_from_ptr_or_opt(py, &_kwargs);
                #from_py_with
            },
            arg_convert,
        );
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
                #pyo3_path::impl_::extract_argument::KeywordOnlyParameterDescription {
                    name: #name,
                    required: #required,
                }
            }
        });

    let num_params = positional_parameter_names.len() + keyword_only_parameters.len();

    let mut option_pos = 0usize;
    let param_conversion = spec
        .signature
        .arguments
        .iter()
        .enumerate()
        .map(|(i, arg)| impl_arg_param(arg, i, &mut option_pos, holders, ctx))
        .collect();

    let args_handler = if spec.signature.python_signature.varargs.is_some() {
        quote! { #pyo3_path::impl_::extract_argument::TupleVarargs }
    } else {
        quote! { #pyo3_path::impl_::extract_argument::NoVarargs }
    };
    let kwargs_handler = if spec.signature.python_signature.kwargs.is_some() {
        quote! { #pyo3_path::impl_::extract_argument::DictVarkeywords }
    } else {
        quote! { #pyo3_path::impl_::extract_argument::NoVarkeywords }
    };

    let cls_name = if let Some(cls) = self_ {
        quote! { ::std::option::Option::Some(<#cls as #pyo3_path::type_object::PyTypeInfo>::NAME) }
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
    (
        quote! {
                const DESCRIPTION: #pyo3_path::impl_::extract_argument::FunctionDescription = #pyo3_path::impl_::extract_argument::FunctionDescription {
                    cls_name: #cls_name,
                    func_name: stringify!(#python_name),
                    positional_parameter_names: &[#(#positional_parameter_names),*],
                    positional_only_parameters: #positional_only_parameters,
                    required_positional_parameters: #required_positional_parameters,
                    keyword_only_parameters: &[#(#keyword_only_parameters),*],
                };
                let mut #args_array = [::std::option::Option::None; #num_params];
                let (_args, _kwargs) = #extract_expression;
                #from_py_with
        },
        param_conversion,
    )
}

fn impl_arg_param(
    arg: &FnArg<'_>,
    pos: usize,
    option_pos: &mut usize,
    holders: &mut Holders,
    ctx: &Ctx,
) -> TokenStream {
    let Ctx { pyo3_path } = ctx;
    let args_array = syn::Ident::new("output", Span::call_site());

    match arg {
        FnArg::Regular(arg) => {
            let from_py_with = format_ident!("from_py_with_{}", pos);
            let arg_value = quote!(#args_array[#option_pos].as_deref());
            *option_pos += 1;
            let tokens = impl_regular_arg_param(arg, from_py_with, arg_value, holders, ctx);
            check_arg_for_gil_refs(tokens, holders.push_gil_refs_checker(arg.ty.span()), ctx)
        }
        FnArg::VarArgs(arg) => {
            let holder = holders.push_holder(arg.name.span());
            let name_str = arg.name.to_string();
            quote_spanned! { arg.name.span() =>
                #pyo3_path::impl_::extract_argument::extract_argument(
                    &_args,
                    &mut #holder,
                    #name_str
                )?
            }
        }
        FnArg::KwArgs(arg) => {
            let holder = holders.push_holder(arg.name.span());
            let name_str = arg.name.to_string();
            quote_spanned! { arg.name.span() =>
                #pyo3_path::impl_::extract_argument::extract_optional_argument(
                    _kwargs.as_deref(),
                    &mut #holder,
                    #name_str,
                    || ::std::option::Option::None
                )?
            }
        }
        FnArg::Py(..) => quote! { py },
        FnArg::CancelHandle(..) => quote! { __cancel_handle },
    }
}

/// Re option_pos: The option slice doesn't contain the py: Python argument, so the argument
/// index and the index in option diverge when using py: Python
pub(crate) fn impl_regular_arg_param(
    arg: &RegularArg<'_>,
    from_py_with: syn::Ident,
    arg_value: TokenStream, // expected type: Option<&'a Bound<'py, PyAny>>
    holders: &mut Holders,
    ctx: &Ctx,
) -> TokenStream {
    let Ctx { pyo3_path } = ctx;
    let pyo3_path = pyo3_path.to_tokens_spanned(arg.ty.span());

    // Use this macro inside this function, to ensure that all code generated here is associated
    // with the function argument
    macro_rules! quote_arg_span {
        ($($tokens:tt)*) => { quote_spanned!(arg.ty.span() => $($tokens)*) }
    }

    let name_str = arg.name.to_string();
    let mut default = arg.default_value.as_ref().map(|expr| quote!(#expr));

    // Option<T> arguments have special treatment: the default should be specified _without_ the
    // Some() wrapper. Maybe this should be changed in future?!
    if arg.option_wrapped_type.is_some() {
        default = Some(default.map_or_else(
            || quote!(::std::option::Option::None),
            |tokens| some_wrap(tokens, ctx),
        ));
    }

    if arg
        .attrs
        .from_py_with
        .as_ref()
        .map(|attr| &attr.value)
        .is_some()
    {
        if let Some(default) = default {
            quote_arg_span! {
                #pyo3_path::impl_::extract_argument::from_py_with_with_default(
                    #arg_value,
                    #name_str,
                    #from_py_with as fn(_) -> _,
                    #[allow(clippy::redundant_closure)]
                    {
                        || #default
                    }
                )?
            }
        } else {
            quote_arg_span! {
                #pyo3_path::impl_::extract_argument::from_py_with(
                    #pyo3_path::impl_::extract_argument::unwrap_required_argument(#arg_value),
                    #name_str,
                    #from_py_with as fn(_) -> _,
                )?
            }
        }
    } else if arg.option_wrapped_type.is_some() {
        let holder = holders.push_holder(arg.name.span());
        quote_arg_span! {
            #pyo3_path::impl_::extract_argument::extract_optional_argument(
                #arg_value,
                &mut #holder,
                #name_str,
                #[allow(clippy::redundant_closure)]
                {
                    || #default
                }
            )?
        }
    } else if let Some(default) = default {
        let holder = holders.push_holder(arg.name.span());
        quote_arg_span! {
            #pyo3_path::impl_::extract_argument::extract_argument_with_default(
                #arg_value,
                &mut #holder,
                #name_str,
                #[allow(clippy::redundant_closure)]
                {
                    || #default
                }
            )?
        }
    } else {
        let holder = holders.push_holder(arg.name.span());
        quote_arg_span! {
            #pyo3_path::impl_::extract_argument::extract_argument(
                #pyo3_path::impl_::extract_argument::unwrap_required_argument(#arg_value),
                &mut #holder,
                #name_str
            )?
        }
    }
}
