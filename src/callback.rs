// Copyright (c) 2017-present PyO3 Project and Contributors

//! Utilities for a Python callable object that invokes a Rust function.

use crate::err::{PyErr, PyResult};
use crate::exceptions::PyOverflowError;
use crate::ffi::{self, Py_hash_t};
use crate::panic::PanicException;
use crate::IntoPyPointer;
use crate::{IntoPy, PyObject, Python};
use std::any::Any;
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

impl PyCallbackOutput for std::os::raw::c_int {
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
    fn convert(self, py: Python<'_>) -> PyResult<Target>;
}

impl<T, E, U> IntoPyCallbackOutput<U> for Result<T, E>
where
    T: IntoPyCallbackOutput<U>,
    E: Into<PyErr>,
{
    #[inline]
    fn convert(self, py: Python<'_>) -> PyResult<U> {
        match self {
            Ok(v) => v.convert(py),
            Err(e) => Err(e.into()),
        }
    }
}

impl<T> IntoPyCallbackOutput<*mut ffi::PyObject> for T
where
    T: IntoPy<PyObject>,
{
    #[inline]
    fn convert(self, py: Python<'_>) -> PyResult<*mut ffi::PyObject> {
        Ok(self.into_py(py).into_ptr())
    }
}

impl IntoPyCallbackOutput<Self> for *mut ffi::PyObject {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<Self> {
        Ok(self)
    }
}

impl IntoPyCallbackOutput<std::os::raw::c_int> for () {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<std::os::raw::c_int> {
        Ok(0)
    }
}

impl IntoPyCallbackOutput<std::os::raw::c_int> for bool {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<std::os::raw::c_int> {
        Ok(self as c_int)
    }
}

impl IntoPyCallbackOutput<()> for () {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<()> {
        Ok(())
    }
}

impl IntoPyCallbackOutput<ffi::Py_ssize_t> for usize {
    #[inline]
    fn convert(self, _py: Python<'_>) -> PyResult<ffi::Py_ssize_t> {
        if self <= (isize::MAX as usize) {
            Ok(self as isize)
        } else {
            Err(PyOverflowError::new_err(()))
        }
    }
}

// Converters needed for `#[pyproto]` implementations

impl IntoPyCallbackOutput<bool> for bool {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<bool> {
        Ok(self)
    }
}

impl IntoPyCallbackOutput<usize> for usize {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<usize> {
        Ok(self)
    }
}

impl<T> IntoPyCallbackOutput<PyObject> for T
where
    T: IntoPy<PyObject>,
{
    #[inline]
    fn convert(self, py: Python<'_>) -> PyResult<PyObject> {
        Ok(self.into_py(py))
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

pub struct HashCallbackOutput(Py_hash_t);

impl IntoPyCallbackOutput<Py_hash_t> for HashCallbackOutput {
    #[inline]
    fn convert(self, _py: Python<'_>) -> PyResult<Py_hash_t> {
        let hash = self.0;
        if hash == -1 {
            Ok(-2)
        } else {
            Ok(hash)
        }
    }
}

impl<T> IntoPyCallbackOutput<HashCallbackOutput> for T
where
    T: WrappingCastTo<Py_hash_t>,
{
    #[inline]
    fn convert(self, _py: Python<'_>) -> PyResult<HashCallbackOutput> {
        Ok(HashCallbackOutput(self.wrapping_cast()))
    }
}

#[doc(hidden)]
#[inline]
pub fn convert<T, U>(py: Python<'_>, value: T) -> PyResult<U>
where
    T: IntoPyCallbackOutput<U>,
{
    value.convert(py)
}

/// Converts the output of std::panic::catch_unwind into a Python function output, either by raising a Python
/// exception or by unwrapping the contained success output.
#[doc(hidden)]
#[inline]
pub fn panic_result_into_callback_output<R>(
    py: Python<'_>,
    panic_result: Result<PyResult<R>, Box<dyn Any + Send + 'static>>,
) -> R
where
    R: PyCallbackOutput,
{
    let py_err = match panic_result {
        Ok(Ok(value)) => return value,
        Ok(Err(py_err)) => py_err,
        Err(payload) => PanicException::from_panic_payload(payload),
    };
    py_err.restore(py);
    R::ERR_VALUE
}
