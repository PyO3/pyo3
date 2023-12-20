use crate::{
    ffi,
    instance::{Py2, Py2Borrowed},
    PyAny, PyResult, Python,
};

mod sealed {
    use super::*;

    pub trait Sealed {}

    impl Sealed for *mut ffi::PyObject {}
}

use sealed::Sealed;

pub(crate) trait FfiPtrExt: Sealed {
    unsafe fn assume_owned_or_err(self, py: Python<'_>) -> PyResult<Py2<'_, PyAny>>;
    unsafe fn assume_owned(self, py: Python<'_>) -> Py2<'_, PyAny>;

    /// Assumes this pointer is borrowed from a parent object.
    ///
    /// Warning: the lifetime `'a` is not bounded by the function arguments; the caller is
    /// responsible to ensure this is tied to some appropriate lifetime.
    unsafe fn assume_borrowed_or_err<'a>(
        self,
        py: Python<'_>,
    ) -> PyResult<Py2Borrowed<'a, '_, PyAny>>;

    /// Same as `assume_borrowed_or_err`, but doesn't fetch an error on NULL.
    unsafe fn assume_borrowed_or_opt<'a>(
        self,
        py: Python<'_>,
    ) -> Option<Py2Borrowed<'a, '_, PyAny>>;

    /// Same as `assume_borrowed_or_err`, but panics on NULL.
    unsafe fn assume_borrowed<'a>(self, py: Python<'_>) -> Py2Borrowed<'a, '_, PyAny>;

    /// Same as `assume_borrowed_or_err`, but does not check for NULL.
    unsafe fn assume_borrowed_unchecked<'a>(self, py: Python<'_>) -> Py2Borrowed<'a, '_, PyAny>;
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

    #[inline]
    unsafe fn assume_borrowed_or_err<'a>(
        self,
        py: Python<'_>,
    ) -> PyResult<Py2Borrowed<'a, '_, PyAny>> {
        Py2Borrowed::from_ptr_or_err(py, self)
    }

    #[inline]
    unsafe fn assume_borrowed_or_opt<'a>(
        self,
        py: Python<'_>,
    ) -> Option<Py2Borrowed<'a, '_, PyAny>> {
        Py2Borrowed::from_ptr_or_opt(py, self)
    }

    #[inline]
    unsafe fn assume_borrowed<'a>(self, py: Python<'_>) -> Py2Borrowed<'a, '_, PyAny> {
        Py2Borrowed::from_ptr(py, self)
    }

    #[inline]
    unsafe fn assume_borrowed_unchecked<'a>(self, py: Python<'_>) -> Py2Borrowed<'a, '_, PyAny> {
        Py2Borrowed::from_ptr_unchecked(py, self)
    }
}
