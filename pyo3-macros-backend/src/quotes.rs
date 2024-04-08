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
        #[allow(unused_imports)]
        use #pyo3_path::impl_::wrap::{IntoPyKind, IntoPyObjectKind};
        let obj = #obj;
        (&obj).into_py_kind().wrap(obj).map_err(::core::convert::Into::<#pyo3_path::PyErr>::into)
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
        #[allow(unused_imports)]
        use #pyo3_path::impl_::wrap::{IntoPyKind, IntoPyObjectKind};
        let result = #result;
        (&result).into_py_kind().map_into_ptr(#py, result)
    }}
}
