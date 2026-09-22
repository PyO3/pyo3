use crate::py_result_ext::PyResultExt;
use crate::types::{ApiObj, FfiObj};
use crate::{
    instance::{Borrowed, Bound},
    PyResult, Python,
};

pub(crate) trait FfiPtrExt {
    type ApiType: ApiObj;

    /// Assumes this pointer carries a Python reference which needs to be decref'd.
    ///
    /// If the pointer is NULL, this function will fetch an error.
    unsafe fn assume_owned_or_err(self, py: Python<'_>) -> PyResult<Bound<'_, Self::ApiType>>;

    /// Same as `assume_owned_or_err`, but doesn't fetch an error on NULL.
    unsafe fn assume_owned_or_opt(self, py: Python<'_>) -> Option<Bound<'_, Self::ApiType>>;

    /// Same as `assume_owned_or_err`, but panics on NULL.
    unsafe fn assume_owned(self, py: Python<'_>) -> Bound<'_, Self::ApiType>;

    /// Same as `assume_owned_or_err`, but does not check for NULL.
    unsafe fn assume_owned_unchecked(self, py: Python<'_>) -> Bound<'_, Self::ApiType>;

    /// Assumes this pointer is borrowed from a parent object.
    ///
    /// Warning: the lifetime `'a` is not bounded by the function arguments; the caller is
    /// responsible to ensure this is tied to some appropriate lifetime.
    unsafe fn assume_borrowed_or_err<'a>(
        self,
        py: Python<'_>,
    ) -> PyResult<Borrowed<'a, '_, Self::ApiType>>;

    /// Same as `assume_borrowed_or_err`, but doesn't fetch an error on NULL.
    unsafe fn assume_borrowed_or_opt<'a>(
        self,
        py: Python<'_>,
    ) -> Option<Borrowed<'a, '_, Self::ApiType>>;

    /// Same as `assume_borrowed_or_err`, but panics on NULL.
    unsafe fn assume_borrowed<'a>(self, py: Python<'_>) -> Borrowed<'a, '_, Self::ApiType>;

    /// Same as `assume_borrowed_or_err`, but does not check for NULL.
    unsafe fn assume_borrowed_unchecked<'a>(
        self,
        py: Python<'_>,
    ) -> Borrowed<'a, '_, Self::ApiType>;
}

impl<T: FfiObj> FfiPtrExt for *mut T {
    type ApiType = <T as crate::types::FfiObj>::ApiType;

    /// # Safety
    ///
    /// see requirements for [`Bound::from_owned_ptr_or_err`]
    #[inline]
    unsafe fn assume_owned_or_err(self, py: Python<'_>) -> PyResult<Bound<'_, Self::ApiType>> {
        // SAFETY: caller upholds requirements
        unsafe { Bound::from_owned_ptr_or_err(py, self.cast()).cast_into_unchecked() }
    }

    /// # Safety
    ///
    /// see requirements for [`Bound::from_owned_ptr_or_opt`]
    #[inline]
    unsafe fn assume_owned_or_opt(self, py: Python<'_>) -> Option<Bound<'_, Self::ApiType>> {
        // SAFETY: caller upholds requirements
        unsafe { Bound::from_owned_ptr_or_opt(py, self.cast()).map(|b| b.cast_into_unchecked()) }
    }

    /// # Safety
    ///
    /// see requirements for [`Bound::from_owned_ptr`]
    #[inline]
    #[track_caller]
    unsafe fn assume_owned(self, py: Python<'_>) -> Bound<'_, Self::ApiType> {
        // SAFETY: caller upholds requirements
        unsafe { Bound::from_owned_ptr(py, self.cast()).cast_into_unchecked() }
    }

    /// # Safety
    ///
    /// see requirements for [`Bound::from_owned_ptr_unchecked`]
    #[inline]
    unsafe fn assume_owned_unchecked(self, py: Python<'_>) -> Bound<'_, Self::ApiType> {
        // SAFETY: caller upholds requirements
        unsafe { Bound::from_owned_ptr_unchecked(py, self.cast()).cast_into_unchecked() }
    }

    /// # Safety
    ///
    /// see requirements for [`Borrowed::from_ptr_or_err`]
    #[inline]
    unsafe fn assume_borrowed_or_err<'a>(
        self,
        py: Python<'_>,
    ) -> PyResult<Borrowed<'a, '_, Self::ApiType>> {
        // SAFETY: caller upholds requirements
        unsafe { Borrowed::from_ptr_or_err(py, self.cast()).map(|b| b.cast_unchecked()) }
    }

    /// # Safety
    ///
    /// see requirements for [`Borrowed::from_ptr_or_opt`]
    #[inline]
    unsafe fn assume_borrowed_or_opt<'a>(
        self,
        py: Python<'_>,
    ) -> Option<Borrowed<'a, '_, Self::ApiType>> {
        // SAFETY: caller upholds requirements
        unsafe { Borrowed::from_ptr_or_opt(py, self.cast()).map(|b| b.cast_unchecked()) }
    }

    /// # Safety
    ///
    /// see requirements for [`Borrowed::from_ptr`]
    #[inline]
    #[track_caller]
    unsafe fn assume_borrowed<'a>(self, py: Python<'_>) -> Borrowed<'a, '_, Self::ApiType> {
        // SAFETY: caller upholds requirements
        unsafe { Borrowed::from_ptr(py, self.cast()).cast_unchecked() }
    }

    /// # Safety
    ///
    /// see requirements for [`Borrowed::from_ptr_unchecked`]
    #[inline]
    unsafe fn assume_borrowed_unchecked<'a>(
        self,
        py: Python<'_>,
    ) -> Borrowed<'a, '_, Self::ApiType> {
        // SAFETY: caller upholds requirements
        unsafe { Borrowed::from_ptr_unchecked(py, self.cast()).cast_unchecked() }
    }
}
