use std::os::raw::c_int;

use crate::err::{error_on_minusone, SignedInteger};
use crate::types::any::PyAnyMethods;
use crate::{ffi, Py2, PyErr, PyResult, Python};

/// Internal helper to convert raw ffi call results such as pointers
/// or integers into safe wrappers.
///
/// `unsafe` to implement because it is highly likely this trait is
/// passed a pointer, and is free to do interpret it as it sees fit.
pub(crate) unsafe trait FromFfiCallResult<'py, RawResult>: Sized {
    fn from_ffi_call_result(py: Python<'py>, raw: RawResult) -> PyResult<Self>;
}

/// For Py2<T>, perform an unchecked downcast to the target type T.
unsafe impl<'py, T> FromFfiCallResult<'py, *mut ffi::PyObject> for Py2<'py, T> {
    fn from_ffi_call_result(py: Python<'py>, raw: *mut ffi::PyObject) -> PyResult<Self> {
        unsafe { Py2::from_owned_ptr_or_err(py, raw).map(|any| any.downcast_into_unchecked()) }
    }
}

unsafe impl<'py, T> FromFfiCallResult<'py, T> for ()
where
    T: SignedInteger,
{
    fn from_ffi_call_result(py: Python<'py>, raw: T) -> PyResult<Self> {
        error_on_minusone(py, raw)
    }
}

unsafe impl<'py, T> FromFfiCallResult<'py, T> for T
where
    T: SignedInteger,
{
    fn from_ffi_call_result(py: Python<'py>, raw: T) -> PyResult<Self> {
        if raw != T::MINUS_ONE {
            Ok(raw)
        } else {
            Err(PyErr::fetch(py))
        }
    }
}

unsafe impl<'py> FromFfiCallResult<'py, c_int> for bool {
    fn from_ffi_call_result(py: Python<'py>, raw: c_int) -> PyResult<Self> {
        match raw {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(PyErr::fetch(py)),
        }
    }
}

/// Convert an isize which is known to be positive to a usize.
#[inline]
pub(crate) fn positive_isize_as_usize(x: isize) -> usize {
    x as usize
}
