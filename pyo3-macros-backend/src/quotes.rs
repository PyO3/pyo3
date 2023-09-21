use proc_macro2::TokenStream;
use quote::quote;

pub(crate) fn some_wrap(obj: TokenStream) -> TokenStream {
    quote! {
        _pyo3::impl_::wrap::SomeWrap::wrap(#obj)
    }
}

pub(crate) fn ok_wrap(obj: TokenStream) -> TokenStream {
    quote! {
        _pyo3::impl_::wrap::OkWrap::wrap(#obj, py)
            .map_err(::core::convert::Into::<_pyo3::PyErr>::into)
    }
}

pub(crate) fn map_result_into_ptr(result: TokenStream) -> TokenStream {
    quote! {
        #result.map(_pyo3::PyObject::into_ptr)
    }
}
