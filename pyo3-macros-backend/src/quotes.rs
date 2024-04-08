use crate::utils::Ctx;
use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn some_wrap(obj: TokenStream, ctx: &Ctx) -> TokenStream {
    let Ctx { pyo3_path } = ctx;
    quote! {
        #pyo3_path::impl_::wrap::SomeWrap::wrap(#obj)
    }
}

pub(crate) fn ok_wrap(obj: TokenStream, ctx: &Ctx) -> TokenStream {
    let Ctx { pyo3_path } = ctx;
    quote! {{
        use #pyo3_path::impl_::wrap::{IntoPyKind, IntoPyObjectKind};
        let obj = #obj;
        (&obj).conversion_kind().wrap(obj).map_err(::core::convert::Into::<#pyo3_path::PyErr>::into)
    }}
}

pub(crate) fn map_result_into_ptr(result: TokenStream, ctx: &Ctx) -> TokenStream {
    let Ctx { pyo3_path } = ctx;
    quote! {{
        use #pyo3_path::impl_::wrap::{IntoPyKind, IntoPyObjectKind};
        let result = #result;
        (&result).conversion_kind().map_into_ptr(py, result)
    }}
}
