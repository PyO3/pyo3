use crate::sealed::Sealed;
use crate::{
    ffi,
    instance::{Borrowed, Bound},
    PyAny, PyResult, Python,
};

pub(crate) trait FfiPtrExt: Sealed {
    /// Assumes this pointer carries a Python reference which needs to be decref'd.
    ///
    /// If the pointer is NULL, this function will fetch an error.
    unsafe fn assume_owned_or_err(self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>>;

    /// Same as `assume_owned_or_err`, but doesn't fetch an error on NULL.
    unsafe fn assume_owned_or_opt(self, py: Python<'_>) -> Option<Bound<'_, PyAny>>;

    /// Same as `assume_owned_or_err`, but panics on NULL.
    unsafe fn assume_owned(self, py: Python<'_>) -> Bound<'_, PyAny>;

    /// Same as `assume_owned_or_err`, but does not check for NULL.
    unsafe fn assume_owned_unchecked(self, py: Python<'_>) -> Bound<'_, PyAny>;

    /// Assumes this pointer is borrowed from a parent object.
    ///
    /// Warning: the lifetime `'a` is not bounded by the function arguments; the caller is
    /// responsible to ensure this is tied to some appropriate lifetime.
    unsafe fn assume_borrowed_or_err<'a>(self, py: Python<'_>)
        -> PyResult<Borrowed<'a, '_, PyAny>>;

    /// Same as `assume_borrowed_or_err`, but doesn't fetch an error on NULL.
    unsafe fn assume_borrowed_or_opt<'a>(self, py: Python<'_>) -> Option<Borrowed<'a, '_, PyAny>>;

    /// Same as `assume_borrowed_or_err`, but panics on NULL.
    unsafe fn assume_borrowed<'a>(self, py: Python<'_>) -> Borrowed<'a, '_, PyAny>;

    /// Same as `assume_borrowed_or_err`, but does not check for NULL.
    unsafe fn assume_borrowed_unchecked<'a>(self, py: Python<'_>) -> Borrowed<'a, '_, PyAny>;
}

impl FfiPtrExt for *mut ffi::PyObject {
    #[inline]
    unsafe fn assume_owned_or_err(self, py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
        unsafe { Bound::from_owned_ptr_or_err(py, self) }
    }

    #[inline]
    unsafe fn assume_owned_or_opt(self, py: Python<'_>) -> Option<Bound<'_, PyAny>> {
        unsafe { Bound::from_owned_ptr_or_opt(py, self) }
    }

    #[inline]
    #[track_caller]
    unsafe fn assume_owned(self, py: Python<'_>) -> Bound<'_, PyAny> {
        unsafe { Bound::from_owned_ptr(py, self) }
    }

    #[inline]
    unsafe fn assume_owned_unchecked(self, py: Python<'_>) -> Bound<'_, PyAny> {
        unsafe { Bound::from_owned_ptr_unchecked(py, self) }
    }

    #[inline]
    unsafe fn assume_borrowed_or_err<'a>(
        self,
        py: Python<'_>,
    ) -> PyResult<Borrowed<'a, '_, PyAny>> {
        unsafe { Borrowed::from_ptr_or_err(py, self) }
    }

    #[inline]
    unsafe fn assume_borrowed_or_opt<'a>(self, py: Python<'_>) -> Option<Borrowed<'a, '_, PyAny>> {
        unsafe { Borrowed::from_ptr_or_opt(py, self) }
    }

    #[inline]
    #[track_caller]
    unsafe fn assume_borrowed<'a>(self, py: Python<'_>) -> Borrowed<'a, '_, PyAny> {
        unsafe { Borrowed::from_ptr(py, self) }
    }

    #[inline]
    unsafe fn assume_borrowed_unchecked<'a>(self, py: Python<'_>) -> Borrowed<'a, '_, PyAny> {
        unsafe { Borrowed::from_ptr_unchecked(py, self) }
    }
}
