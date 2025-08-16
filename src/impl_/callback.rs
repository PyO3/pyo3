//! Utilities for a Python callable object that invokes a Rust function.

use crate::err::{PyErr, PyResult};
use crate::exceptions::PyOverflowError;
use crate::ffi::{self, Py_hash_t};
use crate::{BoundObject, IntoPyObject, Py, PyAny, Python};
use std::ffi::c_int;

/// A type which can be the return type of a python C-API callback
pub trait PyCallbackOutput: Copy {
    /// The error value to return to python if the callback raised an exception
    const ERR_VALUE: Self;
}

impl PyCallbackOutput for *mut ffi::PyObject {
    const ERR_VALUE: Self = std::ptr::null_mut();
}

impl PyCallbackOutput for std::ffi::c_int {
    const ERR_VALUE: Self = -1;
}

impl PyCallbackOutput for ffi::Py_ssize_t {
    const ERR_VALUE: Self = -1;
}

/// Convert the result of callback function into the appropriate return value.
pub trait IntoPyCallbackOutput<'py, Target> {
    fn convert(self, py: Python<'py>) -> PyResult<Target>;
}

impl<'py, T, E, U> IntoPyCallbackOutput<'py, U> for Result<T, E>
where
    T: IntoPyCallbackOutput<'py, U>,
    E: Into<PyErr>,
{
    #[inline]
    fn convert(self, py: Python<'py>) -> PyResult<U> {
        match self {
            Ok(v) => v.convert(py),
            Err(e) => Err(e.into()),
        }
    }
}

impl<'py, T> IntoPyCallbackOutput<'py, *mut ffi::PyObject> for T
where
    T: IntoPyObject<'py>,
{
    #[inline]
    fn convert(self, py: Python<'py>) -> PyResult<*mut ffi::PyObject> {
        self.into_pyobject(py)
            .map(BoundObject::into_ptr)
            .map_err(Into::into)
    }
}

impl IntoPyCallbackOutput<'_, Self> for *mut ffi::PyObject {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<Self> {
        Ok(self)
    }
}

impl IntoPyCallbackOutput<'_, std::ffi::c_int> for () {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<std::ffi::c_int> {
        Ok(0)
    }
}

impl IntoPyCallbackOutput<'_, std::ffi::c_int> for bool {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<std::ffi::c_int> {
        Ok(self as c_int)
    }
}

impl IntoPyCallbackOutput<'_, ()> for () {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<()> {
        Ok(())
    }
}

impl IntoPyCallbackOutput<'_, ffi::Py_ssize_t> for usize {
    #[inline]
    fn convert(self, _py: Python<'_>) -> PyResult<ffi::Py_ssize_t> {
        self.try_into().map_err(|_err| PyOverflowError::new_err(()))
    }
}

// Converters needed for `#[pyproto]` implementations

impl IntoPyCallbackOutput<'_, bool> for bool {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<bool> {
        Ok(self)
    }
}

impl IntoPyCallbackOutput<'_, usize> for usize {
    #[inline]
    fn convert(self, _: Python<'_>) -> PyResult<usize> {
        Ok(self)
    }
}

impl<'py, T> IntoPyCallbackOutput<'py, Py<PyAny>> for T
where
    T: IntoPyObject<'py>,
{
    #[inline]
    fn convert(self, py: Python<'py>) -> PyResult<Py<PyAny>> {
        self.into_pyobject(py)
            .map(BoundObject::into_any)
            .map(BoundObject::unbind)
            .map_err(Into::into)
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

impl IntoPyCallbackOutput<'_, Py_hash_t> for HashCallbackOutput {
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

impl<T> IntoPyCallbackOutput<'_, HashCallbackOutput> for T
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
pub fn convert<'py, T, U>(py: Python<'py>, value: T) -> PyResult<U>
where
    T: IntoPyCallbackOutput<'py, U>,
{
    value.convert(py)
}
