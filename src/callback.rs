//! Utilities for a Python callable object that invokes a Rust function.

use crate::err::{PyErr, PyResult};
use crate::exceptions::{PyOverflowError, PyStopAsyncIteration};
use crate::ffi::{self, Py_hash_t};
use crate::{IntoPy, PyObject, Python};
use std::isize;
use std::os::raw::c_int;
use std::ptr::null_mut;

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

// Autoref-based specialization for handling `__next__` returning `Option`

#[doc(hidden)]
pub struct IterBaseTag;

impl IterBaseTag {
    #[inline]
    pub fn convert<Value, Target>(self, py: Python<'_>, value: Value) -> PyResult<Target>
    where
        Value: IntoPyCallbackOutput<Target>,
    {
        value.convert(py)
    }
}

#[doc(hidden)]
pub trait IterBaseKind {
    fn iter_tag(&self) -> IterBaseTag {
        IterBaseTag
    }
}

impl<Value> IterBaseKind for &Value {}

#[doc(hidden)]
pub struct IterOptionTag;

impl IterOptionTag {
    #[inline]
    pub fn convert<Value>(
        self,
        py: Python<'_>,
        value: Option<Value>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<*mut ffi::PyObject>,
    {
        match value {
            Some(value) => value.convert(py),
            None => Ok(null_mut()),
        }
    }
}

#[doc(hidden)]
pub trait IterOptionKind {
    fn iter_tag(&self) -> IterOptionTag {
        IterOptionTag
    }
}

impl<Value> IterOptionKind for Option<Value> {}

#[doc(hidden)]
pub struct IterResultOptionTag;

impl IterResultOptionTag {
    #[inline]
    pub fn convert<Value>(
        self,
        py: Python<'_>,
        value: PyResult<Option<Value>>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<*mut ffi::PyObject>,
    {
        match value {
            Ok(Some(value)) => value.convert(py),
            Ok(None) => Ok(null_mut()),
            Err(err) => Err(err),
        }
    }
}

#[doc(hidden)]
pub trait IterResultOptionKind {
    fn iter_tag(&self) -> IterResultOptionTag {
        IterResultOptionTag
    }
}

impl<Value> IterResultOptionKind for PyResult<Option<Value>> {}

// Autoref-based specialization for handling `__anext__` returning `Option`

#[doc(hidden)]
pub struct AsyncIterBaseTag;

impl AsyncIterBaseTag {
    #[inline]
    pub fn convert<Value, Target>(self, py: Python<'_>, value: Value) -> PyResult<Target>
    where
        Value: IntoPyCallbackOutput<Target>,
    {
        value.convert(py)
    }
}

#[doc(hidden)]
pub trait AsyncIterBaseKind {
    fn async_iter_tag(&self) -> AsyncIterBaseTag {
        AsyncIterBaseTag
    }
}

impl<Value> AsyncIterBaseKind for &Value {}

#[doc(hidden)]
pub struct AsyncIterOptionTag;

impl AsyncIterOptionTag {
    #[inline]
    pub fn convert<Value>(
        self,
        py: Python<'_>,
        value: Option<Value>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<*mut ffi::PyObject>,
    {
        match value {
            Some(value) => value.convert(py),
            None => Err(PyStopAsyncIteration::new_err(())),
        }
    }
}

#[doc(hidden)]
pub trait AsyncIterOptionKind {
    fn async_iter_tag(&self) -> AsyncIterOptionTag {
        AsyncIterOptionTag
    }
}

impl<Value> AsyncIterOptionKind for Option<Value> {}

#[doc(hidden)]
pub struct AsyncIterResultOptionTag;

impl AsyncIterResultOptionTag {
    #[inline]
    pub fn convert<Value>(
        self,
        py: Python<'_>,
        value: PyResult<Option<Value>>,
    ) -> PyResult<*mut ffi::PyObject>
    where
        Value: IntoPyCallbackOutput<*mut ffi::PyObject>,
    {
        match value {
            Ok(Some(value)) => value.convert(py),
            Ok(None) => Err(PyStopAsyncIteration::new_err(())),
            Err(err) => Err(err),
        }
    }
}

#[doc(hidden)]
pub trait AsyncIterResultOptionKind {
    fn async_iter_tag(&self) -> AsyncIterResultOptionTag {
        AsyncIterResultOptionTag
    }
}

impl<Value> AsyncIterResultOptionKind for PyResult<Option<Value>> {}
