use libc::c_char;
use std::iter::Range;
use std::str::Utf8Error;
use std::mem;
use cstr::CStr;
use ffi;
use python::{Python, ToPythonPointer, PythonObject, PythonObjectWithCheckedDowncast, PythonObjectDowncastError, PythonObjectWithTypeObject};
use err::{self, PyResult};
use super::object::PyObject;
use super::typeobject::PyType;

macro_rules! exc_type(
    ($name:ident, $exc_name:ident) => (
        pyobject_newtype!($name);
        
        impl <'p> PythonObjectWithCheckedDowncast<'p> for $name<'p> {
            #[inline]
            fn downcast_from(obj : PyObject<'p>) -> Result<$name<'p>, PythonObjectDowncastError<'p>> {
                unsafe {
                    if ffi::PyObject_TypeCheck(obj.as_ptr(), ffi::$exc_name as *mut ffi::PyTypeObject) != 0 {
                        Ok(PythonObject::unchecked_downcast_from(obj))
                    } else {
                        Err(PythonObjectDowncastError(obj.python()))
                    }
                }
            }
            
            #[inline]
            fn downcast_borrow_from<'a>(obj : &'a ::objects::object::PyObject<'p>) -> Result<&'a $name<'p>, PythonObjectDowncastError<'p>> {
                unsafe {
                    if ffi::PyObject_TypeCheck(obj.as_ptr(), ffi::$exc_name as *mut ffi::PyTypeObject) != 0 {
                        Ok(PythonObject::unchecked_downcast_borrow_from(obj))
                    } else {
                        Err(PythonObjectDowncastError(obj.python()))
                    }
                }
            }
        }
        
        impl <'p> PythonObjectWithTypeObject<'p> for $name<'p> {
            #[inline]
            fn type_object(py: Python<'p>, _ : Option<&Self>) -> PyType<'p> {
                unsafe { PyType::from_type_ptr(py, ffi::$exc_name as *mut ffi::PyTypeObject) }
            }
        }
    );
    ($name:ident : $base:ident) => (
        exc_type!($name);
    );
);

exc_type!(BaseException, PyExc_BaseException);
exc_type!(Exception, PyExc_Exception);
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

exc_type!(UnicodeDecodeError, PyExc_UnicodeDecodeError);
exc_type!(UnicodeEncodeError, PyExc_UnicodeEncodeError);
exc_type!(UnicodeTranslateError, PyExc_UnicodeTranslateError);

impl<'p> UnicodeDecodeError<'p> {
    pub fn new(py: Python<'p>, encoding: &CStr, input: &[u8], range: Range<usize>, reason: &CStr) -> PyResult<'p, UnicodeDecodeError<'p>> {
        unsafe {
            let input: &[c_char] = mem::transmute(input);
            err::result_cast_from_owned_ptr(py,
                ffi::PyUnicodeDecodeError_Create(encoding.as_ptr(), input.as_ptr(), input.len() as ffi::Py_ssize_t,
                    range.start as ffi::Py_ssize_t, range.end as ffi::Py_ssize_t, reason.as_ptr()))
        }
    }
    
    pub fn new_utf8(py: Python<'p>, input: &[u8], err: Utf8Error) -> PyResult<'p, UnicodeDecodeError<'p>> {
        match err {
            Utf8Error::InvalidByte(pos) =>
                UnicodeDecodeError::new(py, cstr!("utf-8"), input, pos, pos+1, cstr!("invalid byte")),
            Utf8Error::TooShort         =>
                UnicodeDecodeError::new(py, cstr!("utf-8"), input, input.len() - 1, input.len(), cstr!("unexpected end of data")),
        }
    }
}

