// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

//! This module contains the standard python exception types.

use std::os::raw::c_char;
use std::{self, mem, ops};
use std::ffi::CStr;

use ffi;
use pointer::PyObject;
use python::{Python, ToPyPointer};
use err::PyResult;
use super::{PyTuple, PyType};

macro_rules! exc_type(
    ($name:ident, $exc_name:ident) => (
        pub struct $name;

        // pyobject_newtype!($name);

        impl $crate::typeob::PyTypeObject for $name {
            #[inline(always)]
            fn init_type(_py: Python) {}

            #[inline]
            fn type_object<'p>(py: $crate::python::Python<'p>) -> &'p $crate::PyType {
                unsafe { PyType::from_type_ptr(py, ffi::$exc_name as *mut ffi::PyTypeObject) }
            }
        }
    );
);

exc_type!(BaseException, PyExc_BaseException);
exc_type!(Exception, PyExc_Exception);
exc_type!(LookupError, PyExc_LookupError);
exc_type!(AssertionError, PyExc_AssertionError);
exc_type!(AttributeError, PyExc_AttributeError);
exc_type!(BufferError, PyExc_BufferError);
exc_type!(EOFError, PyExc_EOFError);
exc_type!(EnvironmentError, PyExc_EnvironmentError);
exc_type!(FloatingPointError, PyExc_FloatingPointError);
exc_type!(IOError, PyExc_IOError);
exc_type!(ImportError, PyExc_ImportError);
exc_type!(IndexError, PyExc_IndexError);
exc_type!(KeyError, PyExc_KeyError);
exc_type!(KeyboardInterrupt, PyExc_KeyboardInterrupt);
exc_type!(MemoryError, PyExc_MemoryError);
exc_type!(NameError, PyExc_NameError);
exc_type!(NotImplementedError, PyExc_NotImplementedError);
exc_type!(OSError, PyExc_OSError);
exc_type!(OverflowError, PyExc_OverflowError);
exc_type!(ReferenceError, PyExc_ReferenceError);
exc_type!(RuntimeError, PyExc_RuntimeError);
exc_type!(StopIteration, PyExc_StopIteration);
exc_type!(SyntaxError, PyExc_SyntaxError);
exc_type!(SystemError, PyExc_SystemError);
exc_type!(SystemExit, PyExc_SystemExit);
exc_type!(TypeError, PyExc_TypeError);
exc_type!(ValueError, PyExc_ValueError);
#[cfg(target_os="windows")]
exc_type!(WindowsError, PyExc_WindowsError);
exc_type!(ZeroDivisionError, PyExc_ZeroDivisionError);

#[cfg(Py_3)]
exc_type!(BlockingIOError, PyExc_BlockingIOError);
#[cfg(Py_3)]
exc_type!(BrokenPipeError, PyExc_BrokenPipeError);
#[cfg(Py_3)]
exc_type!(ChildProcessError, PyExc_ChildProcessError);
#[cfg(Py_3)]
exc_type!(ConnectionError, PyExc_ConnectionError);
#[cfg(Py_3)]
exc_type!(ConnectionAbortedError, PyExc_ConnectionAbortedError);
#[cfg(Py_3)]
exc_type!(ConnectionRefusedError, PyExc_ConnectionRefusedError);
#[cfg(Py_3)]
exc_type!(ConnectionResetError, PyExc_ConnectionResetError);
#[cfg(Py_3)]
exc_type!(FileExistsError, PyExc_FileExistsError);
#[cfg(Py_3)]
exc_type!(FileNotFoundError, PyExc_FileNotFoundError);
#[cfg(Py_3)]
exc_type!(InterruptedError, PyExc_InterruptedError);
#[cfg(Py_3)]
exc_type!(IsADirectoryError, PyExc_IsADirectoryError);
#[cfg(Py_3)]
exc_type!(NotADirectoryError, PyExc_NotADirectoryError);
#[cfg(Py_3)]
exc_type!(PermissionError, PyExc_PermissionError);
#[cfg(Py_3)]
exc_type!(ProcessLookupError, PyExc_ProcessLookupError);
#[cfg(Py_3)]
exc_type!(TimeoutError, PyExc_TimeoutError);

exc_type!(UnicodeDecodeError, PyExc_UnicodeDecodeError);
exc_type!(UnicodeEncodeError, PyExc_UnicodeEncodeError);
exc_type!(UnicodeTranslateError, PyExc_UnicodeTranslateError);


impl UnicodeDecodeError {

    pub fn new(py: Python, encoding: &CStr, input: &[u8],
               range: ops::Range<usize>, reason: &CStr) -> PyResult<PyObject> {
        unsafe {
            let input: &[c_char] = mem::transmute(input);
            PyObject::from_owned_ptr_or_err(
                py, ffi::PyUnicodeDecodeError_Create(
                    encoding.as_ptr(),
                    input.as_ptr(),
                    input.len() as ffi::Py_ssize_t,
                    range.start as ffi::Py_ssize_t,
                    range.end as ffi::Py_ssize_t,
                    reason.as_ptr()))
        }
    }

    pub fn new_utf8<'p>(py: Python, input: &[u8], err: std::str::Utf8Error)
                        -> PyResult<PyObject>
    {
        let pos = err.valid_up_to();
        UnicodeDecodeError::new(py, cstr!("utf-8"), input, pos .. pos+1, cstr!("invalid utf-8"))
    }
}


impl StopIteration {

    pub fn stop_iteration(_py: Python, args: &PyTuple) {
        unsafe {
            ffi::PyErr_SetObject(
                ffi::PyExc_StopIteration as *mut ffi::PyObject, args.as_ptr());
        }
    }
}
