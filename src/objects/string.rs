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
use libc::c_char;
use ffi;
use python::{Python, PythonObject, PyClone, ToPythonPointer, PythonObjectDowncastError};
use super::{exc, PyObject};
use err::{self, PyResult, PyErr};
use conversion::{FromPyObject, RefFromPyObject, ToPyObject};

/// Represents a Python string.
/// Corresponds to `basestring` in Python 2, and `str` in Python 3.
pub struct PyString(PyObject);

#[cfg(feature="python27-sys")]
pyobject_newtype!(PyString);
#[cfg(feature="python3-sys")]
pyobject_newtype!(PyString, PyUnicode_Check, PyUnicode_Type);

/// Represents a Python byte string.
/// Corresponds to `str` in Python 2, and `bytes` in Python 3.
pub struct PyBytes(PyObject);

pyobject_newtype!(PyBytes, PyBytes_Check, PyBytes_Type);

/// Represents a Python unicode string.
/// Corresponds to `unicode` in Python 2, and `str` in Python 3.
#[cfg(feature="python27-sys")]
pub struct PyUnicode(PyObject);

#[cfg(feature="python27-sys")]
pyobject_newtype!(PyUnicode, PyUnicode_Check, PyUnicode_Type);

/// Represents a Python unicode string.
/// Corresponds to `unicode` in Python 2, and `str` in Python 3.
#[cfg(feature="python3-sys")]
pub use PyString as PyUnicode;

#[cfg(feature="python27-sys")]
impl ::python::PythonObjectWithCheckedDowncast for PyString {
    #[inline]
    fn downcast_from<'p>(py: Python<'p>, obj: PyObject) -> Result<PyString, PythonObjectDowncastError<'p>> {
        if is_base_string(&obj) {
            Ok(PyString(obj))
        } else {
            Err(PythonObjectDowncastError(py))
        }
    }

    #[inline]
    fn downcast_borrow_from<'a, 'p>(py: Python<'p>, obj: &'a PyObject) -> Result<&'a PyString, PythonObjectDowncastError<'p>> {
        unsafe {
            if is_base_string(obj) {
                Ok(::std::mem::transmute(obj))
            } else {
                Err(::python::PythonObjectDowncastError(py))
            }
        }
    }
}

#[cfg(feature="python27-sys")]
#[inline]
fn is_base_string(obj: &PyObject) -> bool {
    unsafe {
        ffi::PyType_FastSubclass(ffi::Py_TYPE(obj.as_ptr()),
            ffi::Py_TPFLAGS_STRING_SUBCLASS | ffi::Py_TPFLAGS_UNICODE_SUBCLASS) != 0
    }
}

#[cfg(feature="python27-sys")]
impl ::python::PythonObjectWithTypeObject for PyString {
    #[inline]
    fn type_object(py: Python) -> super::PyType {
        unsafe { ::objects::typeobject::PyType::from_type_ptr(py, &mut ::ffi::PyBaseString_Type) }
    }
}

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
        #[cfg(feature="python27-sys")]
        fn new_impl(py: Python, s: &str) -> PyString {
            if s.is_ascii() {
                PyBytes::new(py, s.as_bytes()).into_basestring()
            } else {
                PyUnicode::new(py, s).into_basestring()
            }
        }
        #[cfg(feature="python3-sys")]
        fn new_impl(py: Python, s: &str) -> PyString {
            let ptr = s.as_ptr() as *const c_char;
            let len = s.len() as ffi::Py_ssize_t;
            unsafe {
                err::cast_from_owned_ptr_or_panic(py,
                    ffi::PyUnicode_FromStringAndSize(ptr, len))
            }
        }
        new_impl(py, s)
    }

    /// Gets the python string data in its underlying representation.
    ///
    /// For Python 2 byte strings, this function always returns `PyStringData::Utf8`,
    /// even if the bytes are not valid UTF-8.
    /// For unicode strings, returns the underlying representation used by Python.
    pub fn data(&self, py: Python) -> PyStringData {
        self.data_impl(py)
    }

    #[cfg(feature="python27-sys")]
    fn data_impl(&self, py: Python) -> PyStringData {
        if let Ok(bytes) = self.0.cast_as::<PyBytes>(py) {
            PyStringData::Utf8(bytes.data(py))
        } else if let Ok(unicode) = self.0.cast_as::<PyUnicode>(py) {
            unicode.data(py)
        } else {
            panic!("PyString is neither `str` nor `unicode`")
        }
    }

    #[cfg(feature="python3-sys")]
    fn data_impl(&self, py: Python) -> PyStringData {
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

    /// Converts from `PyBytes` to `PyString`.
    /// This method is only available on Python 2.
    #[cfg(feature="python27-sys")]
    #[inline]
    pub fn as_basestring(&self) -> &PyString {
        unsafe { self.0.unchecked_cast_as() }
    }

    /// Converts from `PyBytes` to `PyString`.
    /// This method is only available on Python 2.
    #[cfg(feature="python27-sys")]
    #[inline]
    pub fn into_basestring(self) -> PyString {
        unsafe { self.0.unchecked_cast_into() }
    }
}

#[cfg(feature="python27-sys")]
impl PyUnicode {
    /// Creates a new Python unicode string object.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python, s: &str) -> PyUnicode {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            err::cast_from_owned_ptr_or_panic(py,
                ffi::PyUnicode_FromStringAndSize(ptr, len))
        }
    }

    /// Converts from `PyUnicode` to `PyString`.
    /// This method is only available on Python 2.
    /// (note: on Python 3, `PyUnicode` is a type alias for `PyString`)
    #[inline]
    pub fn as_basestring(&self) -> &PyString {
        unsafe { self.0.unchecked_cast_as() }
    }

    /// Converts from `PyUnicode` to `PyString`.
    /// This method is only available on Python 2.
    /// (note: on Python 3, `PyUnicode` is a type alias for `PyString`)
    #[inline]
    pub fn into_basestring(self) -> PyString {
        unsafe { self.0.unchecked_cast_into() }
    }
    
    /// Gets the python string data in its underlying representation.
    pub fn data(&self, _py: Python) -> PyStringData {
        unsafe {
            let buffer = ffi::PyUnicode_AS_UNICODE(self.as_ptr());
            let length = ffi::PyUnicode_GET_SIZE(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length).into()
        }
    }

    /// Convert the `PyUnicode` into a Rust string.
    ///
    /// Returns a `UnicodeDecodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    pub fn to_string(&self, py: Python) -> PyResult<Cow<str>> {
        self.data(py).to_string(py)
    }

    /// Convert the `PyUnicode` into a Rust string.
    ///
    /// Unpaired surrogates are replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self, py: Python) -> Cow<str> {
        self.data(py).to_string_lossy()
    }
}

/// Converts Rust `str` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for str {
    type ObjectType = PyString;

    #[inline]
    fn to_py_object(&self, py: Python) -> PyString {
        PyString::new(py, self)
    }
}

/// Converts Rust `Cow<str>` to Python object.
/// See `PyString::new` for details on the conversion.
impl <'a> ToPyObject for Cow<'a, str> {
    type ObjectType = PyString;

    #[inline]
    fn to_py_object(&self, py: Python) -> PyString {
        PyString::new(py, self)
    }
}

/// Converts Rust `String` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    type ObjectType = PyString;

    #[inline]
    fn to_py_object(&self, py: Python) -> PyString {
        PyString::new(py, self)
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
/// In Python 2.7, `str` is expected to be UTF-8 encoded.
impl <'source> FromPyObject<'source> for Cow<'source, str> {
    fn extract(py: Python, obj: &'source PyObject) -> PyResult<Self> {
        try!(obj.cast_as::<PyString>(py)).to_string(py)
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
/// In Python 2.7, `str` is expected to be UTF-8 encoded.
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

