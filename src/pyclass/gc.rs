use std::{
    marker::PhantomData,
    os::raw::{c_int, c_void},
};

use crate::{ffi, Py};

/// Error returned by a `__traverse__` visitor implementation.
#[repr(transparent)]
pub struct PyTraverseError(NonZeroCInt);

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
        let ptr = obj.into().map_or_else(std::ptr::null_mut, Py::as_ptr);
        if !ptr.is_null() {
            match NonZeroCInt::new(unsafe { (self.visit)(ptr, self.arg) }) {
                None => Ok(()),
                Some(r) => Err(PyTraverseError(r)),
            }
        } else {
            Ok(())
        }
    }
}

/// Workaround for `NonZero<c_int>` not being available until MSRV 1.79
mod get_nonzero_c_int {
    pub struct GetNonZeroCInt<const WIDTH: usize>();

    pub trait NonZeroCIntType {
        type Type;
    }
    impl NonZeroCIntType for GetNonZeroCInt<16> {
        type Type = std::num::NonZeroI16;
    }
    impl NonZeroCIntType for GetNonZeroCInt<32> {
        type Type = std::num::NonZeroI32;
    }

    pub type Type =
        <GetNonZeroCInt<{ std::mem::size_of::<std::ffi::c_int>() * 8 }> as NonZeroCIntType>::Type;
}

use get_nonzero_c_int::Type as NonZeroCInt;

#[cfg(test)]
mod tests {
    use super::PyVisit;
    use static_assertions::assert_not_impl_any;

    #[test]
    fn py_visit_not_send_sync() {
        assert_not_impl_any!(PyVisit<'_>: Send, Sync);
    }
}
