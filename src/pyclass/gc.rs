use core::{
    ffi::{c_int, c_void},
    marker::PhantomData,
    num::NonZero,
    ops::Deref,
    ptr::NonNull,
};

use crate::{
    ffi,
    impl_::{pycell::PyClassMutability, pyclass::PyClassThreadChecker},
    instance::PyBorrowedUnbound,
    pycell::impl_::{PyClassBorrowChecker, PyClassObjectLayout},
    Py, PyClass,
};

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
#[derive(Clone, Copy)]
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

/// Variant of [`crate::PyClassGuard`] that is used during `__traverse__` calls to
/// ensure that only gc-safe operations are performed on the class instance.
pub(crate) struct PyClassTraverseGuard<'a, T: PyClass> {
    // Caching these two pointers avoids repeated object data lookup, e.g. on drop
    value: NonNull<T>,
    borrow_checker: &'a <T::PyClassMutability as PyClassMutability>::Checker,
    // The original reference which we're fundamentally handling
    phantom: PhantomData<PyBorrowedUnbound<'a, T>>,
}

impl<'a, T: PyClass> PyClassTraverseGuard<'a, T> {
    /// Attempts to create a `PyClassTraverseGuard` from a raw pointer to a class object.
    ///
    /// If the Rust state cannot be safely traversed (e.g. unsendable or currently borrowed),
    /// this returns `None`. This will lead to an incomplete traversal, which is safe but
    /// may leak memory.
    pub(crate) fn try_from_class_object(class_object: PyBorrowedUnbound<'a, T>) -> Option<Self> {
        let contents = T::Layout::contents_during_gc(class_object);

        if !contents.thread_checker.check() {
            return None;
        }

        // This doesn't read from contents because it might need to traverse the ancestry to
        // find the actual borrow checker for the highest mutable base.
        let borrow_checker = T::Layout::borrow_checker_during_gc(class_object);

        borrow_checker.try_borrow().ok().map(|_| {
            // SAFETY: successful borrow implies we have read access to
            // the data, cache a `NonNull` pointer to it which we can use to deref freely
            let value = unsafe { NonNull::from(&*contents.value.get()) };
            Self {
                value,
                borrow_checker,
                phantom: PhantomData,
            }
        })
    }
}

impl<'a, T: PyClass> Deref for PyClassTraverseGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY: we hold a borrow on the underlying data, so we can read it
        unsafe { self.value.as_ref() }
    }
}

impl<'a, T: PyClass> Drop for PyClassTraverseGuard<'a, T> {
    fn drop(&mut self) {
        self.borrow_checker.release_borrow();
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
