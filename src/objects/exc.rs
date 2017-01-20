// Copyright (c) 2015 Daniel Grunwald
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this
// software and associated documentation files (the "Software"), to deal in the Software
// without restriction, including without limitation the rights to use, copy, modify, merge,
// publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons
// to whom the Software is furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED,
// INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE
// FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR
// OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
// DEALINGS IN THE SOFTWARE.

//! This module contains the python exception types.

use libc::c_char;
use std::{self, mem, ops};
use std::ffi::CStr;
use ffi;
use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectDowncastError, PythonObjectWithTypeObject};
use err::{self, PyResult};
use super::object::PyObject;
use super::typeobject::PyType;

macro_rules! exc_type(
    ($name:ident, $exc_name:ident) => (
        pub struct $name(PyObject);

        pyobject_newtype!($name);

        impl PythonObjectWithCheckedDowncast for $name {
            #[inline]
            fn downcast_from<'p>(py: Python<'p>, obj : PyObject)
                -> Result<$name, PythonObjectDowncastError<'p>>
            {
                unsafe {
                    if ffi::PyObject_TypeCheck(obj.as_ptr(), ffi::$exc_name as *mut ffi::PyTypeObject) != 0 {
                        Ok(PythonObject::unchecked_downcast_from(obj))
                    } else {
                        Err(PythonObjectDowncastError(py))
                    }
                }
            }

            #[inline]
            fn downcast_borrow_from<'a, 'p>(py: Python<'p>, obj: &'a PyObject)
                -> Result<&'a $name, PythonObjectDowncastError<'p>>
            {
                unsafe {
                    if ffi::PyObject_TypeCheck(obj.as_ptr(), ffi::$exc_name as *mut ffi::PyTypeObject) != 0 {
                        Ok(PythonObject::unchecked_downcast_borrow_from(obj))
                    } else {
                        Err(PythonObjectDowncastError(py))
                    }
                }
            }
        }

        impl PythonObjectWithTypeObject for $name {
            #[inline]
            fn type_object(py: Python) -> PyType {
                unsafe { PyType::from_type_ptr(py, ffi::$exc_name as *mut ffi::PyTypeObject) }
            }
        }
    );
);

exc_type!(BaseException, PyExc_BaseException);
exc_type!(Exception, PyExc_Exception);
#[cfg(feature="python27-sys")]
exc_type!(StandardError, PyExc_StandardError);
exc_type!(LookupError, PyExc_LookupError);
exc_type!(AssertionError, PyExc_AssertionError);
exc_type!(AttributeError, PyExc_AttributeError);
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
exc_type!(SyntaxError, PyExc_SyntaxError);
exc_type!(SystemError, PyExc_SystemError);
exc_type!(SystemExit, PyExc_SystemExit);
exc_type!(TypeError, PyExc_TypeError);
exc_type!(ValueError, PyExc_ValueError);
#[cfg(target_os="windows")]
exc_type!(WindowsError, PyExc_WindowsError);
exc_type!(ZeroDivisionError, PyExc_ZeroDivisionError);

exc_type!(BufferError, PyExc_BufferError);

exc_type!(UnicodeDecodeError, PyExc_UnicodeDecodeError);
exc_type!(UnicodeEncodeError, PyExc_UnicodeEncodeError);
exc_type!(UnicodeTranslateError, PyExc_UnicodeTranslateError);

impl UnicodeDecodeError {
    pub fn new(py: Python, encoding: &CStr, input: &[u8], range: ops::Range<usize>, reason: &CStr) -> PyResult<UnicodeDecodeError> {
        unsafe {
            let input: &[c_char] = mem::transmute(input);
            err::result_cast_from_owned_ptr(py,
                ffi::PyUnicodeDecodeError_Create(encoding.as_ptr(), input.as_ptr(), input.len() as ffi::Py_ssize_t,
                    range.start as ffi::Py_ssize_t, range.end as ffi::Py_ssize_t, reason.as_ptr()))
        }
    }

    pub fn new_utf8(py: Python, input: &[u8], err: std::str::Utf8Error) -> PyResult<UnicodeDecodeError> {
        let pos = err.valid_up_to();
        UnicodeDecodeError::new(py, cstr!("utf-8"), input, pos .. pos+1, cstr!("invalid utf-8"))
    }
}

