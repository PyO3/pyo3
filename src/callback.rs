// Copyright (c) 2017-present PyO3 Project and Contributors

//! Utilities for a Python callable object that invokes a Rust function.

use crate::err::PyResult;
use crate::exceptions::OverflowError;
use crate::ffi::{self, Py_hash_t};
use crate::IntoPyPointer;
use crate::{IntoPy, PyObject, Python};
use std::isize;
use std::os::raw::c_int;

/// A type which can be the return type of a python C-API callback
pub trait PyCallbackOutput: Copy {
    /// The error value to return to python if the callback raised an exception
    const ERR_VALUE: Self;
}

impl PyCallbackOutput for *mut ffi::PyObject {
    const ERR_VALUE: Self = std::ptr::null_mut();
}

impl PyCallbackOutput for libc::c_int {
    const ERR_VALUE: Self = -1;
}

impl PyCallbackOutput for ffi::Py_ssize_t {
    const ERR_VALUE: Self = -1;
}

impl PyCallbackOutput for () {
    const ERR_VALUE: Self = ();
}

/// Convert the result of callback function into the appropriate return value.
pub trait IntoPyCallbackOutput<Target> {
    fn convert(self, py: Python) -> PyResult<Target>;
}

impl<T, U> IntoPyCallbackOutput<U> for PyResult<T>
where
    T: IntoPyCallbackOutput<U>,
{
    fn convert(self, py: Python) -> PyResult<U> {
        self.and_then(|t| t.convert(py))
    }
}

impl<T> IntoPyCallbackOutput<*mut ffi::PyObject> for T
where
    T: IntoPy<PyObject>,
{
    fn convert(self, py: Python) -> PyResult<*mut ffi::PyObject> {
        Ok(self.into_py(py).into_ptr())
    }
}

impl IntoPyCallbackOutput<Self> for *mut ffi::PyObject {
    fn convert(self, _: Python) -> PyResult<Self> {
        Ok(self)
    }
}

impl IntoPyCallbackOutput<libc::c_int> for () {
    fn convert(self, _: Python) -> PyResult<libc::c_int> {
        Ok(0)
    }
}

impl IntoPyCallbackOutput<libc::c_int> for bool {
    fn convert(self, _: Python) -> PyResult<libc::c_int> {
        Ok(self as c_int)
    }
}

impl IntoPyCallbackOutput<()> for () {
    fn convert(self, _: Python) -> PyResult<()> {
        Ok(())
    }
}

pub struct LenCallbackOutput(pub usize);

impl IntoPyCallbackOutput<ffi::Py_ssize_t> for LenCallbackOutput {
    #[inline]
    fn convert(self, _py: Python) -> PyResult<ffi::Py_ssize_t> {
        if self.0 <= (isize::MAX as usize) {
            Ok(self.0 as isize)
        } else {
            Err(OverflowError::py_err(()))
        }
    }
}

pub trait WrappingCastTo<T> {
    fn wrapping_cast(self) -> T;
}

macro_rules! wrapping_cast {
    ($from:ty, $to:ty) => {
        impl WrappingCastTo<$to> for $from {
            #[inline]
            fn wrapping_cast(self) -> $to {
                self as $to
            }
        }
    };
}
wrapping_cast!(u8, Py_hash_t);
wrapping_cast!(u16, Py_hash_t);
wrapping_cast!(u32, Py_hash_t);
wrapping_cast!(usize, Py_hash_t);
wrapping_cast!(u64, Py_hash_t);
wrapping_cast!(i8, Py_hash_t);
wrapping_cast!(i16, Py_hash_t);
wrapping_cast!(i32, Py_hash_t);
wrapping_cast!(isize, Py_hash_t);
wrapping_cast!(i64, Py_hash_t);

pub struct HashCallbackOutput<T>(pub T);

impl<T> IntoPyCallbackOutput<Py_hash_t> for HashCallbackOutput<T>
where
    T: WrappingCastTo<Py_hash_t>,
{
    #[inline]
    fn convert(self, _py: Python) -> PyResult<Py_hash_t> {
        let hash = self.0.wrapping_cast();
        if hash == -1 {
            Ok(-2)
        } else {
            Ok(hash)
        }
    }
}

#[doc(hidden)]
#[inline]
pub fn convert<T, U>(py: Python, value: T) -> PyResult<U>
where
    T: IntoPyCallbackOutput<U>,
{
    value.convert(py)
}

#[doc(hidden)]
#[inline]
pub fn callback_error<T>() -> T
where
    T: PyCallbackOutput,
{
    T::ERR_VALUE
}

#[doc(hidden)]
pub fn run_callback<T, F>(py: Python, callback: F) -> T
where
    F: FnOnce() -> PyResult<T>,
    T: PyCallbackOutput,
{
    callback().unwrap_or_else(|e| {
        e.restore(py);
        T::ERR_VALUE
    })
}
