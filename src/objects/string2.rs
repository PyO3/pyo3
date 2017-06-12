// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use std::str;
use std::borrow::Cow;
use std::ascii::AsciiExt;
use std::os::raw::c_char;

use ffi;
use err::PyResult;
use pointers::PyPtr;
use python::{Python, ToPyPointer};
use super::{PyObject, PyStringData};

/// Represents a Python string.
pub struct PyString(PyPtr);

pyobject_convert!(PyString);
pyobject_nativetype!(PyString, PyString_Check, PyBaseString_Type);


/// Represents a Python unicode string.
pub struct PyUnicode(PyPtr);

pyobject_convert!(PyUnicode);
pyobject_nativetype!(PyUnicode, PyUnicode_Check, PyUnicode_Type);

/// Represents a Python byte string. Corresponds to `str` in Python 2
pub struct PyBytes(PyPtr);

pyobject_convert!(PyBytes);
pyobject_nativetype!(PyBytes, PyString_Check, PyBaseString_Type);

impl PyString {
    /// Creates a new Python string object.
    ///
    /// This function will create a byte string if the
    /// input string is ASCII-only; and a unicode string otherwise.
    /// Use `PyUnicode::new()` to always create a unicode string.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python, s: &str) -> PyString {
        if s.is_ascii() {
            PyBytes::new(py, s.as_bytes()).into_basestring()
        } else {
            PyUnicode::new(py, s).into_basestring()
        }
    }

    pub fn from_object(py: Python, src: &PyObject,
                       encoding: &str, errors: &str) -> PyResult<PyString> {
        unsafe {
            Ok(PyString(PyPtr::from_owned_ptr_or_err(
                py, ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(), encoding.as_ptr() as *const i8, errors.as_ptr() as *const i8))?
            ))
        }
    }

    /// Gets the python string data in its underlying representation.
    ///
    /// For Python 2 byte strings, this function always returns `PyStringData::Utf8`,
    /// even if the bytes are not valid UTF-8.
    /// For unicode strings, returns the underlying representation used by Python.
    pub fn data(&self, py: Python) -> PyStringData {
        let ob: &PyObject = self.as_ref();
        if let Ok(bytes) = ob.cast_as::<PyBytes>(py) {
            PyStringData::Utf8(bytes.data(py))
        } else if let Ok(unicode) = ob.cast_as::<PyUnicode>(py) {
            unicode.data(py)
        } else {
            panic!("PyString is neither `str` nor `unicode`")
        }
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// On Python 2.7, if the `PyString` refers to a byte string,
    /// it will be decoded using UTF-8.
    ///
    /// Returns a `UnicodeDecodeError` if the input is not valid unicode
    /// (containing unpaired surrogates, or a Python 2.7 byte string that is
    /// not valid UTF-8).
    pub fn to_string(&self, py: Python) -> PyResult<Cow<str>> {
        self.data(py).to_string(py)
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// On Python 2.7, if the `PyString` refers to a byte string,
    /// it will be decoded using UTF-8.
    ///
    /// Unpaired surrogates and (on Python 2.7) invalid UTF-8 sequences are
    /// replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self, py: Python) -> Cow<str> {
        self.data(py).to_string_lossy()
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

    /// Converts from `PyBytes` to `PyString`.
    #[inline]
    pub fn into_basestring(self) -> PyString {
        <PyString as ::PyDowncastInto>::unchecked_downcast_into(self)
    }

    /// Gets the Python string data as byte slice.
    pub fn data(&self, _py: Python) -> &[u8] {
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length)
        }
    }

}

impl PyUnicode {
    /// Creates a new Python unicode string object.
    ///
    /// Panics if out of memory.
    pub fn new(_py: Python, s: &str) -> PyUnicode {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            PyUnicode(PyPtr::from_owned_ptr_or_panic(
                ffi::PyUnicode_FromStringAndSize(ptr, len)))
        }
    }

    pub fn from_object(py: Python, src: &PyObject, encoding: &str, errors: &str)
                       -> PyResult<PyUnicode>
    {
        unsafe {
            Ok(PyUnicode(
                PyPtr::from_owned_ptr_or_err(
                    py, ffi::PyUnicode_FromEncodedObject(
                        src.as_ptr(),
                        encoding.as_ptr() as *const i8,
                        errors.as_ptr() as *const i8))?))
        }
    }

    /// Converts from `PyUnicode` to `PyString`.
    #[inline]
    pub fn into_basestring(self) -> PyString {
        <PyString as ::PyDowncastInto>::unchecked_downcast_into(self)
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

