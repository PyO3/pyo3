// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use std::str;
use std::borrow::Cow;
use std::os::raw::c_char;

use ffi;
use err::PyResult;
use object::PyObject;
use instance::{Py, PyObjectWithToken};
use python::{Python, ToPyPointer};
use objectprotocol::ObjectProtocol;
use super::{PyObjectRef, PyStringData};

/// Represents a Python `string`.
pub struct PyString(PyObject);

pyobject_convert!(PyString);
pyobject_nativetype!(PyString, PyBaseString_Type, PyBaseString_Check);

/// Represents a Python `unicode string`.
pub struct PyUnicode(PyObject);

pyobject_convert!(PyUnicode);
pyobject_nativetype!(PyUnicode, PyUnicode_Type, PyUnicode_Check);

/// Represents a Python `byte` string. Corresponds to `str` in Python 2
pub struct PyBytes(PyObject);

pyobject_convert!(PyBytes);
pyobject_nativetype!(PyBytes, PyBaseString_Type, PyString_Check);


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
                src.py(), ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(),
                    encoding.as_ptr() as *const c_char,
                    errors.as_ptr() as *const c_char))?
            )
        }
    }

    /// Gets the python string data in its underlying representation.
    ///
    /// For Python 2 byte strings, this function always returns `PyStringData::Utf8`,
    /// even if the bytes are not valid UTF-8.
    /// For unicode strings, returns the underlying representation used by Python.
    pub fn data(&self) -> PyStringData {
        if let Ok(bytes) = self.cast_as::<PyBytes>() {
            PyStringData::Utf8(bytes.data())
        } else if let Ok(unicode) = self.cast_as::<PyUnicode>() {
            unicode.data()
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
        self.data().to_string(self.py())
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// On Python 2.7, if the `PyString` refers to a byte string,
    /// it will be decoded using UTF-8.
    ///
    /// Unpaired surrogates and (on Python 2.7) invalid UTF-8 sequences are
    /// replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self) -> Cow<str> {
        self.data().to_string_lossy()
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
        unsafe {
            Py::from_owned_ptr_or_panic(ffi::PyBytes_FromStringAndSize(ptr, len))
        }
    }

    /// Gets the Python string data as byte slice.
    pub fn data(&self) -> &[u8] {
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
    pub fn new(_py: Python, s: &str) -> Py<PyUnicode> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            Py::from_owned_ptr_or_panic(ffi::PyUnicode_FromStringAndSize(ptr, len))
        }
    }

    pub fn from_object(src: &PyObjectRef, encoding: &str, errors: &str) -> PyResult<Py<PyUnicode>>
    {
        unsafe {
            Ok(Py::from_owned_ptr_or_err(
                src.py(), ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(),
                    encoding.as_ptr() as *const c_char,
                    errors.as_ptr() as *const c_char))?)
        }
    }

    /// Gets the python string data in its underlying representation.
    pub fn data(&self) -> PyStringData {
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
    pub fn to_string(&self) -> PyResult<Cow<str>> {
        self.data().to_string(self.py())
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates are replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self) -> Cow<str> {
        self.data().to_string_lossy()
    }
}

/// Converts from `PyBytes` to `PyString`.
impl std::convert::From<Py<PyBytes>> for Py<PyString> {
    #[inline]
    fn from(ob: Py<PyBytes>) -> Py<PyString> {
        unsafe{std::mem::transmute(ob)}
    }
}

/// Converts from `PyUnicode` to `PyString`.
impl std::convert::From<Py<PyUnicode>> for Py<PyString> {
    #[inline]
    fn from(ob: Py<PyUnicode>) -> Py<PyString> {
        unsafe{std::mem::transmute(ob)}
    }
}


#[cfg(test)]
mod test {
    use python::Python;
    use instance::AsPyRef;
    use conversion::{ToPyObject, FromPyObject};

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
}
