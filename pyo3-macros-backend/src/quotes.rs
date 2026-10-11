use crate::utils::Ctx;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

pub(crate) fn some_wrap(obj: TokenStream, ctx: &Ctx) -> TokenStream {
    let Ctx { pyo3_path, .. } = ctx;
    quote! {
        #pyo3_path::impl_::wrap::SomeWrap::wrap(#obj)
    }
}

/// Fused return-value conversion: wraps a (possibly `Result`) return value and converts it
/// into a `PyResult<*mut ffi::PyObject>` in a single call. `obj` must be an identifier
/// bound to the function's return value (it is evaluated twice).
pub(crate) fn wrap_into_ptr(obj: TokenStream, ctx: &Ctx) -> TokenStream {
    let Ctx {
        pyo3_path,
        output_span,
    } = ctx;
    let pyo3_path = pyo3_path.to_tokens_spanned(*output_span);
    let py = syn::Ident::new("py", proc_macro2::Span::call_site());
    quote_spanned! { *output_span =>
        #pyo3_path::impl_::wrap::converter(&#obj).wrap_into_ptr(#py, #obj)
    }
}

/// As `wrap_into_ptr`, but produces a `PyResult<Py<PyAny>>`. The Python token expression
/// is supplied by the caller.
pub(crate) fn wrap_into_pyobject(obj: TokenStream, py: TokenStream, ctx: &Ctx) -> TokenStream {
    let Ctx {
        pyo3_path,
        output_span,
    } = ctx;
    let pyo3_path = pyo3_path.to_tokens_spanned(*output_span);
    quote_spanned! { *output_span =>
        #pyo3_path::impl_::wrap::converter(&#obj).wrap_into_pyobject(#py, #obj)
    }
}
