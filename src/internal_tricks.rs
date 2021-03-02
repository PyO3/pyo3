use std::ffi::{CStr, CString};
use std::marker::PhantomData;
use std::rc::Rc;

/// A marker type that makes the type !Send.
/// Temporal hack until https://github.com/rust-lang/rust/issues/13231 is resolved.
pub(crate) type Unsendable = PhantomData<Rc<()>>;

pub struct PrivateMarker;

macro_rules! private_decl {
    () => {
        /// This trait is private to implement; this method exists to make it
        /// impossible to implement outside the crate.
        fn __private__(&self) -> crate::internal_tricks::PrivateMarker;
    };
}

macro_rules! private_impl {
    () => {
        #[doc(hidden)]
        fn __private__(&self) -> crate::internal_tricks::PrivateMarker {
            crate::internal_tricks::PrivateMarker
        }
    };
}

macro_rules! pyo3_exception {
    ($doc: expr, $name: ident, $base: ty) => {
        #[doc = $doc]
        #[repr(transparent)]
        #[allow(non_camel_case_types)]
        pub struct $name($crate::PyAny);

        $crate::impl_exception_boilerplate!($name);

        $crate::create_exception_type_object!(pyo3_runtime, $name, $base);
    };
}

#[derive(Debug)]
pub(crate) struct NulByteInString(pub(crate) &'static str);

pub(crate) fn extract_cstr_or_leak_cstring(
    src: &'static str,
    err_msg: &'static str,
) -> Result<&'static CStr, NulByteInString> {
    CStr::from_bytes_with_nul(src.as_bytes())
        .or_else(|_| {
            CString::new(src.as_bytes()).map(|c_string| &*Box::leak(c_string.into_boxed_c_str()))
        })
        .map_err(|_| NulByteInString(err_msg))
}
