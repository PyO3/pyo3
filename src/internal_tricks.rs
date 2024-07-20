use crate::ffi::{Py_ssize_t, PY_SSIZE_T_MAX};
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

        $crate::create_exception_type_object!(pyo3_runtime, $name, $base, Some($doc));
    };
}

/// Convert an usize index into a Py_ssize_t index, clamping overflow to
/// PY_SSIZE_T_MAX.
pub(crate) fn get_ssize_index(index: usize) -> Py_ssize_t {
    index.min(PY_SSIZE_T_MAX as usize) as Py_ssize_t
}

// TODO: use ptr::from_ref on MSRV 1.76
#[inline]
pub(crate) const fn ptr_from_ref<T>(t: &T) -> *const T {
    t as *const T
}

// TODO: use ptr::from_mut on MSRV 1.76
#[inline]
pub(crate) fn ptr_from_mut<T>(t: &mut T) -> *mut T {
    t as *mut T
}
