use core::{
    ffi::{c_int, c_void},
    marker::PhantomData,
    num::NonZero,
};

use crate::{ffi, Py};

/// Error returned by a `__traverse__` visitor implementation.
#[repr(transparent)]
pub struct PyTraverseError(NonZero<c_int>);

impl PyTraverseError {
    /// Returns the error code.
    pub(crate) fn into_inner(self) -> c_int {
        self.0.into()
    }
}

/// Object visitor for GC.
#[derive(Clone)]
pub struct PyVisit<'a> {
    visit: ffi::visitproc,
    arg: *mut c_void,
    /// Prevents the `PyVisit` from outliving the `__traverse__` call.
    _guard: PhantomData<&'a ()>,
}

impl PyVisit<'_> {
    /// Visit `obj`.
    ///
    /// Note: `obj` accepts a variety of types, including
    /// - `&Py<T>`
    /// - `&Option<Py<T>>`
    /// - `Option<&Py<T>>`
    pub fn call<'a, T, U: 'a>(&self, obj: T) -> Result<(), PyTraverseError>
    where
        T: Into<Option<&'a Py<U>>>,
    {
        let ptr = obj.into().map_or_else(core::ptr::null_mut, Py::as_ptr);
        if !ptr.is_null() {
            // SAFETY: `PyVisit` is only created during a `tp_traverse` call and
            // cannot outlive it, so `visit` and `arg` are still the pair the
            // interpreter passed in; `ptr` comes from `Py::as_ptr`, so it is a
            // valid object pointer.
            let retval = unsafe { (self.visit)(ptr, self.arg) };
            make_traverse_result(retval)
        } else {
            Ok(())
        }
    }

    /// # Safety
    ///
    /// The caller must pass a `visit` and `arg` pair that are guaranteed to be valid for
    /// the inferred lifetime `'a`.
    pub(crate) unsafe fn new(visit: ffi::visitproc, arg: *mut c_void) -> Self {
        Self {
            visit,
            arg,
            _guard: PhantomData,
        }
    }
}

/// Cast the return value of a `visitproc` to a `Result<(), PyTraverseError>`.
///
/// This should optimize to a no-op in release builds due to the 0 niche in `PyTraverseError`.
#[inline]
pub(crate) fn make_traverse_result(retval: c_int) -> Result<(), PyTraverseError> {
    match NonZero::new(retval) {
        None => Ok(()),
        Some(r) => Err(PyTraverseError(r)),
    }
}

#[cfg(test)]
mod tests {
    use super::PyVisit;
    use static_assertions::assert_not_impl_any;

    #[test]
    fn py_visit_not_send_sync() {
        assert_not_impl_any!(PyVisit<'_>: Send, Sync);
    }
}
