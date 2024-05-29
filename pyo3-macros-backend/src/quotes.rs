use crate::utils::Ctx;
use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

pub(crate) fn some_wrap(obj: TokenStream, ctx: &Ctx) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;
    quote! {
        #pyo3_path::impl_::wrap::SomeWrap::wrap(#obj)
    }
}

pub(crate) fn ok_wrap(obj: TokenStream, ctx: &Ctx, span: Span) -> TokenStream {
    let pyo3_path = ctx.pyo3_path.to_tokens_spanned(span);
    quote_spanned! { span=>
        #pyo3_path::impl_::wrap::OkWrap::wrap(#obj)
            .map_err(::core::convert::Into::<#pyo3_path::PyErr>::into)
    }
}

pub(crate) fn map_result_into_ptr(result: TokenStream, ctx: &Ctx, span: Span) -> TokenStream {
    let pyo3_path = ctx.pyo3_path.to_tokens_spanned(span);
    quote_spanned! { span => #pyo3_path::impl_::wrap::map_result_into_ptr(py, #result) }
}
