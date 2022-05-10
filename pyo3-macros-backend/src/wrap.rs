use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse::Parse, Token};

pub struct WrapPyFunctionArgs {
    function: syn::Path,
    comma_and_arg: Option<(Token![,], syn::Expr)>,
}

impl Parse for WrapPyFunctionArgs {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let function = input.parse()?;
        let comma_and_arg = if !input.is_empty() {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };
        Ok(Self {
            function,
            comma_and_arg,
        })
    }
}

pub fn wrap_pyfunction_impl(args: WrapPyFunctionArgs) -> TokenStream {
    let WrapPyFunctionArgs {
        function,
        comma_and_arg,
    } = args;
    if let Some((_, arg)) = comma_and_arg {
        quote! { #function::wrap(#function::DEF, #arg) }
    } else {
        quote! { &|arg| #function::wrap(#function::DEF, arg) }
    }
}

pub fn wrap_pymodule_impl(module_path: syn::Path) -> syn::Result<TokenStream> {
    Ok(quote! {
        &|py| unsafe { #module_path::DEF.make_module(py).expect("failed to wrap pymodule") }
    })
}
