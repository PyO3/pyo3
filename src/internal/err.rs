use std::mem::ManuallyDrop;

use crate::{ffi, PyErr, Python};

/// Denotes that an error occurred and the Python interpreter has an error set
/// in the thread state.
///
/// While this type is alive, it is unsafe to do most Python C API calls.
///
/// Dropping this type will clear the error in the Python interpreter. This
/// type also has a conversion to `PyErr` to fetch the error.
pub struct ErrorAlreadySet<'py>(Python<'py>);

impl<'py> ErrorAlreadySet<'py> {
    /// # Safety
    /// - Caller is responsible to ensure that no Python C APIs are called while
    ///   this type is alive (except for those specifically for error handling).
    #[inline]
    pub unsafe fn new(py: Python<'py>) -> Self {
        Self(py)
    }
}

impl Drop for ErrorAlreadySet<'_> {
    #[inline]
    fn drop(&mut self) {
        // SAFETY: `self.0` proves the thread is attached to the interpreter.
        unsafe { ffi::PyErr_Clear() }
    }
}

impl From<ErrorAlreadySet<'_>> for PyErr {
    #[inline]
    fn from(err: ErrorAlreadySet<'_>) -> Self {
        // Avoid calling `PyErr_Clear` after fetching
        let err = ManuallyDrop::new(err);
        PyErr::fetch(err.0)
    }
}
