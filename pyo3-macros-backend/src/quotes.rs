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
        {
            #[allow(unused_imports)]
            use #pyo3_path::impl_::wrap::{IntoPyKind, IntoPyObjectKind};
            #[allow(clippy::needless_borrow)]
            (&&&obj).conversion_kind().wrap(obj).map_err(::core::convert::Into::<#pyo3_path::PyErr>::into)
        }
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
        {
            #[allow(unused_imports)]
            use #pyo3_path::impl_::wrap::{IntoPyKind, IntoPyObjectKind, IntoPyNoneKind};
            #[allow(clippy::needless_borrow)]
            (&&&result).conversion_kind().map_into_ptr(#py, result)
        }
    }}
}
