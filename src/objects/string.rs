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

use std;
use std::{mem, str, char};
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::os::raw::c_char;
use ffi;
use python::{Python, PythonObject, PyClone, ToPythonPointer, PythonObjectDowncastError};
use super::{exc, PyObject};
use err::{self, PyResult, PyErr};
use conversion::{FromPyObject, RefFromPyObject, ToPyObject};

/// Represents a Python string.
pub struct PyString(PyObject);

pyobject_newtype!(PyString, PyUnicode_Check, PyUnicode_Type);

/// Represents a Python byte string.
pub struct PyBytes(PyObject);

pyobject_newtype!(PyBytes, PyBytes_Check, PyBytes_Type);


/// Enum of possible Python string representations.
#[derive(Clone, Copy, Debug)]
pub enum PyStringData<'a> {
    Latin1(&'a [u8]),
    Utf8(&'a [u8]),
    Utf16(&'a [u16]),
    Utf32(&'a [u32])
}

impl <'a> From<&'a str> for PyStringData<'a> {
    #[inline]
    fn from(val: &'a str) -> PyStringData<'a> {
        PyStringData::Utf8(val.as_bytes())
    }
}

impl <'a> From<&'a [u16]> for PyStringData<'a> {
    #[inline]
    fn from(val: &'a [u16]) -> PyStringData<'a> {
        PyStringData::Utf16(val)
    }
}

impl <'a> From<&'a [u32]> for PyStringData<'a> {
    #[inline]
    fn from(val: &'a [u32]) -> PyStringData<'a> {
        PyStringData::Utf32(val)
    }
}

impl <'a> PyStringData<'a> {
    /// Convert the Python string data to a Rust string.
    ///
    /// For UTF-8 and ASCII-only latin-1, returns a borrow into the original string data.
    /// For Latin-1, UTF-16 and UTF-32, returns an owned string.
    ///
    /// Fails with UnicodeDecodeError if the string data isn't valid in its encoding.
    pub fn to_string(self, py: Python) -> PyResult<Cow<'a, str>> {
        match self {
            PyStringData::Utf8(data) => {
                match str::from_utf8(data) {
                    Ok(s) => Ok(Cow::Borrowed(s)),
                    Err(e) => Err(PyErr::from_instance(py, try!(exc::UnicodeDecodeError::new_utf8(py, data, e))))
                }
            }
            PyStringData::Latin1(data) => {
                if data.iter().all(|&b| b.is_ascii()) {
                    Ok(Cow::Borrowed(unsafe { str::from_utf8_unchecked(data) }))
                } else {
                    Ok(Cow::Owned(data.iter().map(|&b| b as char).collect()))
                }
            },
            PyStringData::Utf16(data) => {
                fn utf16_bytes(input: &[u16]) -> &[u8] {
                    unsafe { mem::transmute(input) }
                }
                match String::from_utf16(data) {
                    Ok(s) => Ok(Cow::Owned(s)),
                    Err(_) => Err(PyErr::from_instance(py,
                        try!(exc::UnicodeDecodeError::new(py, cstr!("utf-16"),
                            utf16_bytes(data), 0 .. 2*data.len(), cstr!("invalid utf-16")))
                    ))
                }
            },
            PyStringData::Utf32(data) => {
                fn utf32_bytes(input: &[u32]) -> &[u8] {
                    unsafe { mem::transmute(input) }
                }
                match data.iter().map(|&u| char::from_u32(u)).collect() {
                    Some(s) => Ok(Cow::Owned(s)),
                    None => Err(PyErr::from_instance(py,
                        try!(exc::UnicodeDecodeError::new(py, cstr!("utf-32"),
                            utf32_bytes(data), 0 .. 4*data.len(), cstr!("invalid utf-32")))
                    ))
                }
            }
        }
    }

    /// Convert the Python string data to a Rust string.
    ///
    /// Returns a borrow into the original string data if possible.
    ///
    /// Data that isn't valid in its encoding will be replaced
    /// with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(self) -> Cow<'a, str> {
        match self {
            PyStringData::Utf8(data) => String::from_utf8_lossy(data),
            PyStringData::Latin1(data) => {
                if data.iter().all(|&b| b.is_ascii()) {
                    Cow::Borrowed(unsafe { str::from_utf8_unchecked(data) })
                } else {
                    Cow::Owned(data.iter().map(|&b| b as char).collect())
                }
            },
            PyStringData::Utf16(data) => {
                Cow::Owned(String::from_utf16_lossy(data))
            },
            PyStringData::Utf32(data) => {
                Cow::Owned(data.iter()
                    .map(|&u| char::from_u32(u).unwrap_or('\u{FFFD}'))
                    .collect())
            }
        }
    }
}

impl PyString {
    /// Creates a new Python string object.
    ///
    /// On Python 2.7, this function will create a byte string if the
    /// input string is ASCII-only; and a unicode string otherwise.
    /// Use `PyUnicode::new()` to always create a unicode string.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python, s: &str) -> PyString {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            err::cast_from_owned_ptr_or_panic(py,
                                              ffi::PyUnicode_FromStringAndSize(ptr, len))
        }
    }

    pub fn from_object(py: Python, src: &PyObject, encoding: &str, errors: &str) -> PyResult<PyString> {
        unsafe {
            err::result_cast_from_owned_ptr(
                py, ffi::PyUnicode_FromEncodedObject(
                    src.as_ptr(), encoding.as_ptr() as *const i8, errors.as_ptr() as *const i8))
        }
    }

    /// Gets the python string data in its underlying representation.
    pub fn data(&self, py: Python) -> PyStringData {
        // TODO: return the original representation instead
        // of forcing the UTF-8 representation to be created.
        unsafe {
            let mut size : ffi::Py_ssize_t = mem::uninitialized();
            let data = ffi::PyUnicode_AsUTF8AndSize(self.as_ptr(), &mut size) as *const u8;
            if data.is_null() {
                PyErr::fetch(py).print(py);
                panic!("PyUnicode_AsUTF8AndSize failed");
            }
            PyStringData::Utf8(std::slice::from_raw_parts(data, size as usize))
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
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self, py: Python) -> Cow<str> {
        self.data(py).to_string_lossy()
    }
}

impl PyBytes {
    /// Creates a new Python byte string object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python, s: &[u8]) -> PyBytes {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            err::cast_from_owned_ptr_or_panic(py,
                ffi::PyBytes_FromStringAndSize(ptr, len))
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
}

/// Converts Rust `str` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for str {
    #[inline]
    fn to_py_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into_object()
    }
}

/// Converts Rust `Cow<str>` to Python object.
/// See `PyString::new` for details on the conversion.
impl <'a> ToPyObject for Cow<'a, str> {
    #[inline]
    fn to_py_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into_object()
    }
}

/// Converts Rust `String` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_py_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into_object()
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl <'source> FromPyObject<'source> for Cow<'source, str> {
    fn extract(py: Python, obj: &'source PyObject) -> PyResult<Self> {
        try!(obj.cast_as::<PyString>(py)).to_string(py)
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
impl <'source> FromPyObject<'source> for String {
    fn extract(py: Python, obj: &'source PyObject) -> PyResult<Self> {
        obj.extract::<Cow<str>>(py).map(Cow::into_owned)
    }
}

impl RefFromPyObject for str {
    fn with_extracted<F, R>(py: Python, obj: &PyObject, f: F) -> PyResult<R>
        where F: FnOnce(&str) -> R
    {
        let s = try!(obj.extract::<Cow<str>>(py));
        Ok(f(&s))
    }
}

#[cfg(test)]
mod test {
    use python::{Python, PythonObject};
    use conversion::{ToPyObject, RefFromPyObject};

    #[test]
    fn test_non_bmp() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "\u{1F30F}";
        let py_string = s.to_py_object(py).into_object();
        assert_eq!(s, py_string.extract::<String>(py).unwrap());
    }

    #[test]
    fn test_extract_str() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "Hello Python";
        let py_string = s.to_py_object(py).into_object();
        let mut called = false;
        RefFromPyObject::with_extracted(py, &py_string,
            |s2: &str| {
                assert_eq!(s, s2);
                called = true;
            }).unwrap();
        assert!(called);
    }
}

