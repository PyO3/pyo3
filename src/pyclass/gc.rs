// TODO https://github.com/PyO3/pyo3/issues/5487
#![allow(clippy::undocumented_unsafe_blocks)]

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
#[derive(Clone)]
pub struct PyVisit<'a> {
    pub(crate) visit: ffi::visitproc,
    pub(crate) arg: *mut c_void,
    /// Prevents the `PyVisit` from outliving the `__traverse__` call.
    pub(crate) _guard: PhantomData<&'a ()>,
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
            match NonZero::new(unsafe { (self.visit)(ptr, self.arg) }) {
                None => Ok(()),
                Some(r) => Err(PyTraverseError(r)),
            }
        } else {
            Ok(())
        }
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
