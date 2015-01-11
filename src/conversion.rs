use libc::c_char;
use std;
use ffi;
use python::{Python, PythonObject, PythonObjectWithCheckedDowncast, ToPythonPointer};
use objects::{PyObject, PyBool, exc};
use err::{self, PyErr, PyResult};

/// ToPyObject is implemented for types that can be converted into a python object.
/// The goal is to allow methods that take a python object to take anything that
/// can be converted into a python object.
/// For example, compare calling the following method signatures:
///   fn m1(o: &PyObject) {}
///   fn m2<O>(o: &O) where O : ToPyObject {}
///
///   let o: &PyObject = ...;
///   m1(o);
///   m2(o);
///
///   let p: PyPtr<PyObject> = ...;
///   m1(*p)
///   m2(p)
///
///   let i: i32 = ...;
///   m1(*try!(i.to_py_object(py)))
///   m2(i)
pub trait ToPyObject<'p> : Sized {
    type ObjectType : PythonObject<'p> = PyObject<'p>;
    
    fn to_py_object(self, py: Python<'p>) -> PyResult<'p, Self::ObjectType>;
    
    #[inline]
    fn with_borrowed_ptr<F, R>(self, py: Python<'p>, f: F) -> PyResult<'p, R> where F: FnOnce(*mut ffi::PyObject) -> PyResult<'p, R> {
        let obj = try!(self.to_py_object(py));
        f(ToPythonPointer::as_ptr(&obj))
    }
    
    // FFI functions that accept a borrowed reference will use:
    //   input.with_borrowed_ptr(|obj| ffi::Call(obj.as_ptr())
    // 1) input is &PyObject
    //   -> with_py_object() just forwards to the closure
    // 2) input is PyPtr<PyObject>
    //   -> to_py_object() is no-op; FFI call happens; PyPtr::drop() calls Py_DECREF()
    // 3) input is &str, int, ...
    //   -> to_py_object() allocates new python object; FFI call happens; PyPtr::drop() calls Py_DECREF()
    
    // FFI functions that steal a reference will use:
    //   let input = try!(input.to_py_object()); ffi::Call(input.into_owned_ptr())
    // 1) input is &PyObject
    //   -> to_py_object() calls Py_INCREF
    // 2) input is PyPtr<PyObject>
    //   -> to_py_object() is no-op
    // 3) input is &str, int, ...
    //   -> to_py_object() allocates new python object
}

/// FromPyObject is implemented by various types that can be extracted from a python object.
pub trait FromPyObject<'p, 's> {
    fn from_py_object(s: &'s PyObject<'p>) -> PyResult<'p, Self>;
}

// PyObject, PyModule etc.
// We support FromPyObject and ToPyObject for owned python references.
// This allows using existing python objects in code that generically expects a value
// convertible to a python object.

impl <'p, T> ToPyObject<'p> for T where T: PythonObject<'p> {
    type ObjectType = T;
    
    #[inline]
    fn to_py_object(self, py: Python<'p>) -> PyResult<'p, T> {
        Ok(self)
    }
}

impl <'p, 's, T> FromPyObject<'p, 's> for T where T: PythonObjectWithCheckedDowncast<'p> {
    #[inline]
    fn from_py_object(s : &'s PyObject<'p>) -> PyResult<'p, T> {
        Ok(try!(s.clone().cast_into()))
    }
}

// &PyObject, &PyModule etc.
// We support FromPyObject and ToPyObject for borrowed python references.
// This allows using existing python objects in code that generically expects a value
// convertible to a python object.

impl <'p, 's, T> ToPyObject<'p> for &'s T where T : PythonObject<'p> {
    type ObjectType = T;
    
    #[inline]
    fn to_py_object(self, py: Python<'p>) -> PyResult<'p, T> {
        Ok((*self).clone())
    }
    
    #[inline]
    fn with_borrowed_ptr<F, R>(self, py: Python<'p>, f: F) -> PyResult<'p, R>
        where F: FnOnce(*mut ffi::PyObject) -> PyResult<'p, R>
    {
        // Avoid unnecessary Py_INCREF/Py_DECREF pair
        f(self.as_ptr())
    }
}

// bool


impl <'p> ToPyObject<'p> for bool {
    type ObjectType = PyBool<'p>;
    
    #[inline]
    fn to_py_object(self, py: Python<'p>) -> PyResult<'p, PyBool<'p>> {
        Ok(PyBool::get(py, self))
    }
    
    #[inline]
    fn with_borrowed_ptr<F, R>(self, py: Python<'p>, f: F) -> PyResult<'p, R>
        where F: FnOnce(*mut ffi::PyObject) -> PyResult<'p, R>
    {
        // Avoid unnecessary Py_INCREF/Py_DECREF pair
        f(unsafe { if self { ffi::Py_True() } else { ffi::Py_False() } })
    }
}

impl <'p, 'a> FromPyObject<'p, 'a> for bool {
    fn from_py_object(s: &'a PyObject<'p>) -> PyResult<'p, bool> {
        Ok(try!(s.clone().cast_into::<PyBool>()).is_true())
    }
}

// Strings.
// When converting strings to/from python, we need to copy the string data.
// This means we can implement ToPyObject for str, but FromPyObject only for String.
impl <'p, 's> ToPyObject<'p> for &'s str {
    type ObjectType = PyObject<'p>;
    
    fn to_py_object(self, py : Python<'p>) -> PyResult<'p, PyObject<'p>> {
        let ptr : *const c_char = self.as_ptr() as *const _;
        let len : ffi::Py_ssize_t = std::num::from_uint(self.len()).unwrap();
        unsafe {
            use std::ascii::AsciiExt;
            let obj = if self.is_ascii() {
                ffi::PyString_FromStringAndSize(ptr, len)
            } else {
                ffi::PyUnicode_FromStringAndSize(ptr, len)
            };
            err::result_from_owned_ptr(py, obj)
        }
    }
}

impl <'p, 's> FromPyObject<'p, 's> for &'s str {
    fn from_py_object(o: &'s PyObject<'p>) -> PyResult<'p, &'s str> {
        let s = try!(string_as_slice(o));
        match std::str::from_utf8(s) {
            Ok(s) => Ok(s),
            Err(err) => Err(PyErr::new(try!(exc::UnicodeDecodeError::new_utf8(o.python(), s, err))))
        }
    }
}

pub fn string_as_slice<'a, 'p>(s: &'a PyObject<'p>) -> PyResult<'p, &'a [u8]> {
    unsafe {
        let mut buffer : *mut c_char = std::mem::uninitialized();
        let mut length : ffi::Py_ssize_t = std::mem::uninitialized();
        if ffi::PyString_AsStringAndSize(s.as_ptr(), &mut buffer, &mut length) == 1 {
            Err(PyErr::fetch(s.python()))
        } else {
            let buffer = buffer as *const u8; // TODO see std::ffi
            Ok(std::slice::from_raw_buf(std::mem::copy_lifetime(s, &buffer), length as usize))
        }
    }
}

