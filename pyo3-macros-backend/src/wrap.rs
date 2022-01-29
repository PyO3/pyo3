use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse::Parse, spanned::Spanned, Ident, Token};

pub struct WrapPyFunctionArgs {
    function: syn::Path,
    comma_and_arg: Option<(Token![,], syn::Expr)>,
}

impl Parse for WrapPyFunctionArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
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

pub fn wrap_pymodule_impl(mut module_path: syn::Path) -> syn::Result<TokenStream> {
    let span = module_path.span();
    let last_segment = module_path
        .segments
        .last_mut()
        .ok_or_else(|| err_spanned!(span => "expected non-empty path"))?;

    last_segment.ident = module_def_ident(&last_segment.ident);

    Ok(quote! {

        &|py| unsafe { #module_path.make_module(py).expect("failed to wrap pymodule") }
    })
}

pub(crate) fn module_def_ident(name: &Ident) -> Ident {
    format_ident!("__PYO3_PYMODULE_DEF_{}", name.to_string().to_uppercase())
}
