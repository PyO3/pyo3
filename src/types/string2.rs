// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use std::borrow::Cow;
use std::os::raw::c_char;
use std::str;

use super::PyObjectRef;
use err::{PyErr, PyResult};
use ffi;
use instance::{Py, PyObjectWithGIL};
use object::PyObject;
use objectprotocol::ObjectProtocol;
use python::{Python, ToPyPointer};
use types::exceptions;

/// Represents a Python `string`.
#[repr(transparent)]
pub struct PyString(PyObject);

pyobject_native_type!(PyString, ffi::PyBaseString_Type, ffi::PyBaseString_Check);

/// Represents a Python `unicode string`.
#[repr(transparent)]
pub struct PyUnicode(PyObject);

pyobject_native_type!(PyUnicode, ffi::PyUnicode_Type, ffi::PyUnicode_Check);

/// Represents a Python `byte` string. Corresponds to `str` in Python 2
#[repr(transparent)]
pub struct PyBytes(PyObject);

pyobject_native_type!(PyBytes, ffi::PyBaseString_Type, ffi::PyString_Check);

impl PyString {
    /// Creates a new Python string object.
    ///
    /// This function will create a byte string if the
    /// input string is ASCII-only; and a unicode string otherwise.
    /// Use `PyUnicode::new()` to always create a unicode string.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python, s: &str) -> Py<PyString> {
        if s.is_ascii() {
            PyBytes::new(py, s.as_bytes()).into()
        } else {
            PyUnicode::new(py, s).into()
        }
    }

    pub fn from_object(src: &PyObjectRef, encoding: &str, errors: &str) -> PyResult<Py<PyString>> {
        unsafe {
            Ok(Py::from_owned_ptr_or_err(
                src.py(),
                ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(),
                    encoding.as_ptr() as *const c_char,
                    errors.as_ptr() as *const c_char,
                ),
            )?)
        }
    }

    /// Get the Python string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        if let Ok(bytes) = self.cast_as::<PyBytes>() {
            bytes.as_bytes()
        } else if let Ok(unicode) = self.cast_as::<PyUnicode>() {
            unicode.as_bytes()
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
    pub fn to_string(&self) -> PyResult<Cow<str>> {
        match std::str::from_utf8(self.as_bytes()) {
            Ok(s) => Ok(Cow::Borrowed(s)),
            Err(e) => Err(PyErr::from_instance(
                exceptions::UnicodeDecodeError::new_utf8(self.py(), self.as_bytes(), e)?,
            )),
        }
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// On Python 2.7, if the `PyString` refers to a byte string,
    /// it will be decoded using UTF-8.
    ///
    /// Unpaired surrogates and (on Python 2.7) invalid UTF-8 sequences are
    /// replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.as_bytes())
    }
}

impl PyBytes {
    /// Creates a new Python byte string object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new(_py: Python, s: &[u8]) -> Py<PyBytes> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe { Py::from_owned_ptr_or_panic(ffi::PyBytes_FromStringAndSize(ptr, len)) }
    }

    /// Get the Python string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            debug_assert!(!buffer.is_null());
            std::slice::from_raw_parts(buffer, length)
        }
    }
}

impl PyUnicode {
    /// Creates a new Python unicode string object.
    ///
    /// Panics if out of memory.
    pub fn new(_py: Python, s: &str) -> Py<PyUnicode> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe { Py::from_owned_ptr_or_panic(ffi::PyUnicode_FromStringAndSize(ptr, len)) }
    }

    pub fn from_object(src: &PyObjectRef, encoding: &str, errors: &str) -> PyResult<Py<PyUnicode>> {
        unsafe {
            Ok(Py::from_owned_ptr_or_err(
                src.py(),
                ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(),
                    encoding.as_ptr() as *const c_char,
                    errors.as_ptr() as *const c_char,
                ),
            )?)
        }
    }

    /// Get the Python string as a byte slice.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        unsafe {
            // PyUnicode_AsUTF8String would return null if the pointer did not reference a valid
            // unicode object, but because we have a valid PyUnicode, assume success
            let data: Py<PyBytes> =
                Py::from_owned_ptr(ffi::PyUnicode_AsUTF8String(self.0.as_ptr()));
            let buffer = ffi::PyBytes_AsString(data.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(data.as_ptr()) as usize;
            debug_assert!(!buffer.is_null());
            std::slice::from_raw_parts(buffer, length)
        }
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Returns a `UnicodeDecodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    pub fn to_string(&self) -> PyResult<Cow<str>> {
        match std::str::from_utf8(self.as_bytes()) {
            Ok(s) => Ok(Cow::Borrowed(s)),
            Err(e) => Err(PyErr::from_instance(
                exceptions::UnicodeDecodeError::new_utf8(self.py(), self.as_bytes(), e)?,
            )),
        }
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates are replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.as_bytes())
    }
}

/// Converts from `PyBytes` to `PyString`.
impl std::convert::From<Py<PyBytes>> for Py<PyString> {
    #[inline]
    fn from(ob: Py<PyBytes>) -> Py<PyString> {
        unsafe { std::mem::transmute(ob) }
    }
}

/// Converts from `PyUnicode` to `PyString`.
impl std::convert::From<Py<PyUnicode>> for Py<PyString> {
    #[inline]
    fn from(ob: Py<PyUnicode>) -> Py<PyString> {
        unsafe { std::mem::transmute(ob) }
    }
}

#[cfg(test)]
mod test {
    use super::PyString;
    use conversion::{FromPyObject, PyTryFrom, ToPyObject};
    use instance::AsPyRef;
    use object::PyObject;
    use python::Python;
    use std::borrow::Cow;

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

        let s2: &str = FromPyObject::extract(py_string.as_ref(py)).unwrap();
        assert_eq!(s, s2);
    }

    #[test]
    fn test_as_bytes() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "ascii 🐈";
        let obj: PyObject = PyString::new(py, s).into();
        let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
        assert_eq!(s.as_bytes(), py_string.as_bytes());
    }

    #[test]
    fn test_to_string_ascii() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "ascii";
        let obj: PyObject = PyString::new(py, s).into();
        let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
        assert!(py_string.to_string().is_ok());
        assert_eq!(Cow::Borrowed(s), py_string.to_string().unwrap());
    }

    #[test]
    fn test_to_string_unicode() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "哈哈🐈";
        let obj: PyObject = PyString::new(py, s).into();
        let py_string = <PyString as PyTryFrom>::try_from(obj.as_ref(py)).unwrap();
        assert!(py_string.to_string().is_ok());
        assert_eq!(Cow::Borrowed(s), py_string.to_string().unwrap());
    }
}
