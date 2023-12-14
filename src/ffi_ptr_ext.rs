use crate::{ffi, instance::Py2, PyAny, PyResult, Python};

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for *mut ffi::PyObject {}
}

use sealed::Sealed;

pub(crate) trait FfiPtrExt: Sealed {
    unsafe fn assume_owned_or_err(self, py: Python<'_>) -> PyResult<Py2<'_, PyAny>>;
    unsafe fn assume_owned(self, py: Python<'_>) -> Py2<'_, PyAny>;
}

impl FfiPtrExt for *mut ffi::PyObject {
    #[inline]
    unsafe fn assume_owned_or_err(self, py: Python<'_>) -> PyResult<Py2<'_, PyAny>> {
        Py2::from_owned_ptr_or_err(py, self)
    }

    #[inline]
    unsafe fn assume_owned(self, py: Python<'_>) -> Py2<'_, PyAny> {
        Py2::from_owned_ptr(py, self)
    }
}
