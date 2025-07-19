use crate::utils::Ctx;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

pub(crate) fn some_wrap(obj: TokenStream, ctx: &Ctx) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;
    quote! {
        #pyo3_path::impl_::wrap::SomeWrap::wrap(#obj)
    }
}

pub(crate) fn ok_wrap(obj: TokenStream, ctx: &Ctx) -> TokenStream {
    let Ctx {
        pyo3_path,
        output_span,
    } = ctx;
    let pyo3_path = pyo3_path.to_tokens_spanned(*output_span);
    quote_spanned! { *output_span => {
        let obj = #obj;
        #[allow(clippy::useless_conversion)]
        #pyo3_path::impl_::wrap::converter(&obj).wrap(obj).map_err(::core::convert::Into::<#pyo3_path::PyErr>::into)
    }}
}

pub(crate) fn map_result_into_ptr(result: TokenStream, ctx: &Ctx) -> TokenStream {
    let Ctx {
        pyo3_path,
        output_span,
    } = ctx;
    let pyo3_path = pyo3_path.to_tokens_spanned(*output_span);
    let py = syn::Ident::new("py", proc_macro2::Span::call_site());
    quote_spanned! { *output_span => {
        let result = #result;
        #pyo3_path::impl_::wrap::converter(&result).map_into_ptr(#py, result)
    }}
}
