// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use std::str;
use std::borrow::Cow;
use std::os::raw::c_char;

use ffi;
use err::PyResult;
use pointers::PyPtr;
use python::{Python, ToPyPointer};
use super::{PyObject, PyStringData};

/// Represents a Python string. Corresponds to `unicode` in Python 2
pub struct PyString(PyPtr);

pyobject_convert!(PyString);
pyobject_nativetype!(PyString, PyUnicode_Check, PyUnicode_Type);

/// Represents a Python byte string. Corresponds to `str` in Python 2
pub struct PyBytes(PyPtr);

pyobject_convert!(PyBytes);
pyobject_nativetype!(PyBytes, PyString_Check, PyBaseString_Type);


impl PyBytes {
    /// Creates a new Python byte string object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new(_py: Python, s: &[u8]) -> PyBytes {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            PyBytes(PyPtr::from_owned_ptr_or_panic(
                ffi::PyBytes_FromStringAndSize(ptr, len)))
        }
    }

    /// Gets the Python string data as byte slice.
    pub fn data(&self, _py: Python) -> &[u8] {
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length)
        }
    }

    #[inline]
    pub fn is_base_string(obj: &PyObject) -> bool {
        unsafe {
            ffi::PyType_FastSubclass(
                ffi::Py_TYPE(obj.as_ptr()),
                ffi::Py_TPFLAGS_STRING_SUBCLASS | ffi::Py_TPFLAGS_UNICODE_SUBCLASS) != 0
        }
    }
}

impl PyString {
    /// Creates a new Python unicode string object.
    ///
    /// Panics if out of memory.
    pub fn new(_py: Python, s: &str) -> PyString {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            PyString(PyPtr::from_owned_ptr_or_panic(
                ffi::PyUnicode_FromStringAndSize(ptr, len)))
        }
    }

    pub fn from_object(py: Python, src: &PyObject, encoding: &str, errors: &str)
                       -> PyResult<PyString>
    {
        unsafe {
            Ok(PyString(
                PyPtr::from_owned_ptr_or_err(
                    py, ffi::PyUnicode_FromEncodedObject(
                        src.as_ptr(),
                        encoding.as_ptr() as *const i8,
                        errors.as_ptr() as *const i8))?))
        }
    }

    /// Converts from `PyString` to `PyBytes`.
    #[inline]
    pub fn into_bytes(self) -> PyBytes {
        <PyBytes as ::PyDowncastInto>::unchecked_downcast_into(self)
    }
    
    /// Gets the python string data in its underlying representation.
    pub fn data(&self, _py: Python) -> PyStringData {
        unsafe {
            let buffer = ffi::PyUnicode_AS_UNICODE(self.as_ptr());
            let length = ffi::PyUnicode_GET_SIZE(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length).into()
        }
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Returns a `UnicodeDecodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    pub fn to_string(&self, py: Python) -> PyResult<Cow<str>> {
        self.data(py).to_string(py)
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates are replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self, py: Python) -> Cow<str> {
        self.data(py).to_string_lossy()
    }
}

#[cfg(test)]
mod test {
    use python::Python;
    use conversion::{ToPyObject, RefFromPyObject};

    #[test]
    fn test_non_bmp() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "\u{1F30F}";
        let py_string = s.to_object(py);
        assert_eq!(s, py_string.extract::<String>(py).unwrap());
    }

    #[test]
    fn test_extract_str() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "Hello Python";
        let py_string = s.to_object(py);
        let mut called = false;
        RefFromPyObject::with_extracted(py, &py_string,
            |s2: &str| {
                assert_eq!(s, s2);
                called = true;
            }).unwrap();
        assert!(called);
    }
}

