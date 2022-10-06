// Copyright (c) 2017-present PyO3 Project and Contributors

//! Utilities for a Python callable object that invokes a Rust function.

use crate::err::{PyErr, PyResult};
use crate::exceptions::PyOverflowError;
use crate::ffi::{self, Py_hash_t};
use crate::impl_::panic::PanicTrap;
use crate::panic::PanicException;
use crate::{GILPool, IntoPyPointer};
use crate::{IntoPy, Py, PyAny, PyObject, Python};
use std::any::Any;
use std::os::raw::c_int;
use std::panic::UnwindSafe;
use std::{isize, panic};

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

pub trait OkWrap<T> {
    type Error;
    fn wrap(self, py: Python<'_>) -> Result<Py<PyAny>, Self::Error>;
}

impl<T> OkWrap<T> for T
where
    T: IntoPy<PyObject>,
{
    type Error = PyErr;
    fn wrap(self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        Ok(self.into_py(py))
    }
}

impl<T, E> OkWrap<T> for Result<T, E>
where
    T: IntoPy<PyObject>,
{
    type Error = E;
    fn wrap(self, py: Python<'_>) -> Result<Py<PyAny>, Self::Error> {
        self.map(|o| o.into_py(py))
    }
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

/// Use this macro for all internal callback functions which Python will call.
///
/// It sets up the GILPool and converts the output into a Python object. It also restores
/// any python error returned as an Err variant from the body.
///
/// Finally, any panics inside the callback body will be caught and translated into PanicExceptions.
///
/// # Safety
/// This macro assumes the GIL is held. (It makes use of unsafe code, so usage of it is only
/// possible inside unsafe blocks.)
#[doc(hidden)]
#[macro_export]
macro_rules! callback_body {
    ($py:ident, $body:expr) => {
        $crate::callback::handle_panic(|$py| $crate::callback::convert($py, $body))
    };
}

/// Variant of the above which does not perform the callback conversion. This allows the callback
/// conversion to be done manually in the case where lifetimes might otherwise cause issue.
///
/// For example this pyfunction:
///
/// ```no_compile
/// fn foo(&self) -> &Bar {
///     &self.bar
/// }
/// ```
///
/// It is wrapped in proc macros with handle_panic like so:
///
/// ```no_compile
/// pyo3::callback::handle_panic(|_py| {
///     let _slf = #slf;
///     pyo3::callback::convert(_py, #foo)
/// })
/// ```
///
/// If callback_body was used instead:
///
/// ```no_compile
/// pyo3::callback_body!(py, {
///     let _slf = #slf;
///     #foo
/// })
/// ```
///
/// Then this will fail to compile, because the result of #foo borrows _slf, but _slf drops when
/// the block passed to the macro ends.
#[doc(hidden)]
#[inline]
pub unsafe fn handle_panic<F, R>(body: F) -> R
where
    F: for<'py> FnOnce(Python<'py>) -> PyResult<R> + UnwindSafe,
    R: PyCallbackOutput,
{
    let trap = PanicTrap::new("uncaught panic at ffi boundary");
    let pool = GILPool::new();
    let py = pool.python();
    let out = panic_result_into_callback_output(
        py,
        panic::catch_unwind(move || -> PyResult<_> { body(py) }),
    );
    trap.disarm();
    out
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
