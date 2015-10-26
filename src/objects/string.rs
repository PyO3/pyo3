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
use std::str;
use std::ascii::AsciiExt;
use std::borrow::Cow;
use libc::c_char;
use ffi;
use python::{Python, PythonObject, PyClone, ToPythonPointer};
use super::{exc, PyObject};
use err::{self, PyResult, PyErr};
use conversion::{ExtractPyObject, ToPyObject};

/// Represents a Python byte string.
/// Corresponds to `str` in Python 2, and `bytes` in Python 3.
pub struct PyBytes(PyObject);

/// Represents a Python unicode string.
/// Corresponds to `unicode` in Python 2, and `str` in Python 3.
pub struct PyUnicode(PyObject);

pyobject_newtype!(PyBytes, PyBytes_Check, PyBytes_Type);
pyobject_newtype!(PyUnicode, PyUnicode_Check, PyUnicode_Type);

#[cfg(feature="python27-sys")]
pub use PyBytes as PyString;
#[cfg(feature="python3-sys")]
pub use PyUnicode as PyString;

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
    pub fn as_slice(&self, _py: Python) -> &[u8] {
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length)
        }
    }
}

impl PyUnicode {
    /// Creates a new unicode string object from the Rust string.
    pub fn new(py: Python, s: &str) -> PyUnicode {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            err::cast_from_owned_ptr_or_panic(py,
                ffi::PyUnicode_FromStringAndSize(ptr, len))
        }
    }

    /* Note: 'as_slice removed temporarily, we need to reconsider
    // whether we really should expose the platform-dependent Py_UNICODE to user code.
    #[cfg(feature="python27-sys")]
    pub fn as_slice(&self, py: Python) -> &[ffi::Py_UNICODE] {
        unsafe {
            let buffer = ffi::PyUnicode_AS_UNICODE(self.as_ptr()) as *const _;
            let length = ffi::PyUnicode_GET_SIZE(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length)
        }
    }*/

    /// Convert the `PyUnicode` into a rust string.
    ///
    /// Returns a `UnicodeDecodeError` if the input contains invalid code points.
    #[cfg(feature="python27-sys")]
    pub fn to_string(&self, py: Python) -> PyResult<Cow<str>> {
        let bytes: PyBytes = unsafe {
            try!(err::result_cast_from_owned_ptr(py, ffi::PyUnicode_AsUTF8String(self.as_ptr())))
        };
        match str::from_utf8(bytes.as_slice(py)) {
            Ok(s) => Ok(Cow::Owned(s.to_owned())),
            Err(e) => Err(PyErr::from_instance(py, try!(exc::UnicodeDecodeError::new_utf8(py, bytes.as_slice(py), e))))
        }
    }

    /// Convert the `PyUnicode` into a rust string.
    ///
    /// Any invalid code points are replaced with U+FFFD REPLACEMENT CHARACTER.
    #[cfg(feature="python27-sys")]
    pub fn to_string_lossy(&self, py: Python) -> Cow<str> {
        // TODO: test how this function handles lone surrogates or otherwise invalid code points
        let bytes: PyBytes = unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyUnicode_AsUTF8String(self.as_ptr()))
                .ok().expect("Error in PyUnicode_AsUTF8String")
        };
        Cow::Owned(String::from_utf8_lossy(bytes.as_slice(py)).into_owned())
    }

    #[cfg(feature="python3-sys")]
    fn to_utf8_bytes(&self, py: Python) -> PyResult<&[u8]> {
        unsafe {
            let mut length = 0;
            let data = ffi::PyUnicode_AsUTF8AndSize(self.as_ptr(), &mut length);
            if data.is_null() {
                Err(PyErr::fetch(py))
            } else {
                Ok(std::slice::from_raw_parts(data as *const u8, length as usize))
            }
        }
    }

    /// Convert the `PyUnicode` into a rust string.
    ///
    /// Returns a `UnicodeDecodeError` if the input contains invalid code points.
    #[cfg(feature="python3-sys")]
    pub fn to_string(&self, py: Python) -> PyResult<Cow<str>> {
        let bytes = try!(self.to_utf8_bytes(py));
        match str::from_utf8(bytes) {
            Ok(s) => Ok(Cow::Borrowed(s)),
            Err(e) => Err(PyErr::from_instance(py, try!(exc::UnicodeDecodeError::new_utf8(py, bytes, e))))
        }
    }

    /// Convert the `PyUnicode` into a rust string.
    ///
    /// Any invalid code points are replaced with U+FFFD REPLACEMENT CHARACTER.
    #[cfg(feature="python3-sys")]
    pub fn to_string_lossy(&self, py: Python) -> Cow<str> {
        let bytes = self.to_utf8_bytes(py).expect("Error in PyUnicode_AsUTF8AndSize");
        String::from_utf8_lossy(bytes)
    }
}

// On PyString (i.e. PyBytes in 2.7, PyUnicode otherwise), put static methods
// for extraction as Cow<str>:
impl PyString {
    /// Extract a rust string from the Python object.
    ///
    /// In Python 2.7, accepts both byte strings and unicode strings.
    /// Byte strings will be decoded using UTF-8.
    ///
    /// In Python 3.x, accepts unicode strings only.
    ///
    /// Returns `TypeError` if the input is not one of the accepted types.
    /// Returns `UnicodeDecodeError` if the input is not valid unicode.
    #[cfg(feature="python27-sys")]
    pub fn extract<'a>(py: Python, o: &'a PyObject) -> PyResult<Cow<'a, str>> {
        if let Ok(s) = o.cast_as::<PyBytes>(py) {
            s.to_string(py)
        } else if let Ok(u) = o.cast_as::<PyUnicode>(py) {
            u.to_string(py)
        } else {
            Err(PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None))
        }
    }

    /// Extract a rust string from the Python object.
    ///
    /// In Python 2.7, accepts both byte strings and unicode strings.
    /// Byte strings will be decoded using UTF-8.
    ///
    /// In Python 3.x, accepts unicode strings only.
    ///
    /// Returns `TypeError` if the input is not one of the accepted types.
    /// Any invalid code points are replaced with U+FFFD REPLACEMENT CHARACTER.
    #[cfg(feature="python27-sys")]
    pub fn extract_lossy<'a>(py: Python, o: &'a PyObject) -> PyResult<Cow<'a, str>> {
        if let Ok(s) = o.cast_as::<PyBytes>(py) {
            Ok(s.to_string_lossy(py))
        } else if let Ok(u) = o.cast_as::<PyUnicode>(py) {
            Ok(u.to_string_lossy(py))
        } else {
            Err(PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None))
        }
    }

    /// Extract a rust string from the Python object.
    ///
    /// In Python 2.7, accepts both byte strings and unicode strings.
    /// Byte strings will be decoded using UTF-8.
    ///
    /// In Python 3.x, accepts unicode strings only.
    ///
    /// Returns `TypeError` if the input is not one of the accepted types.
    /// Returns `UnicodeDecodeError` if the input is not valid unicode.
    #[cfg(feature="python3-sys")]
    pub fn extract<'a>(py: Python, o: &'a PyObject) -> PyResult<Cow<'a, str>> {
        if let Ok(u) = o.cast_as::<PyUnicode>(py) {
            u.to_string(py)
        } else {
            Err(PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None))
        }
    }

    /// Extract a rust string from the Python object.
    ///
    /// In Python 2.7, accepts both byte strings and unicode strings.
    /// Byte strings will be decoded using UTF-8.
    ///
    /// In Python 3.x, accepts unicode strings only.
    ///
    /// Returns `TypeError` if the input is not one of the accepted types.
    /// Any invalid code points are replaced with U+FFFD REPLACEMENT CHARACTER.
    #[cfg(feature="python3-sys")]
    pub fn extract_lossy<'a>(py: Python, o: &'a PyObject) -> PyResult<Cow<'a, str>> {
        if let Ok(u) = o.cast_as::<PyUnicode>(py) {
            Ok(u.to_string_lossy(py))
        } else {
            Err(PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None))
        }
    }

    // In Python 2.7, PyBytes serves as PyString, so it should offer the
    // same to_string and to_string_lossy functions as PyUnicode:

    /// Convert the `PyString` into a rust string.
    ///
    /// In Python 2.7, `PyString` is a byte string and will be decoded using UTF-8.
    /// In Python 3.x, `PyString` is a unicode string.
    ///
    /// Returns a `UnicodeDecodeError` if the input is not valid unicode.
    #[cfg(feature="python27-sys")]
    pub fn to_string(&self, py: Python) -> PyResult<Cow<str>> {
        match str::from_utf8(self.as_slice(py)) {
            Ok(s) => Ok(Cow::Borrowed(s)),
            Err(e) => Err(PyErr::from_instance(py, try!(exc::UnicodeDecodeError::new_utf8(py, self.as_slice(py), e))))
        }
    }

    /// Convert the `PyString` into a rust string.
    ///
    /// In Python 2.7, `PyString` is a byte string and will be decoded using UTF-8.
    /// In Python 3.x, `PyString` is a unicode string.
    ///
    /// Any invalid UTF-8 sequences are replaced with U+FFFD REPLACEMENT CHARACTER.
    #[cfg(feature="python27-sys")]
    pub fn to_string_lossy(&self, py: Python) -> Cow<str> {
        String::from_utf8_lossy(self.as_slice(py))
    }
}

// When converting strings to/from Python, we need to copy the string data.
// This means we can implement ToPyObject for str, but FromPyObject only for (Cow)String.

/// Converts rust `str` to Python object:
/// ASCII-only strings are converted to Python `str` objects;
/// other strings are converted to Python `unicode` objects.
///
/// Note that `str::ObjectType` differs based on Python version:
/// In Python 2.7, it is `PyObject` (`object` is the common base class of `str` and `unicode`).
/// In Python 3.x, it is `PyUnicode`.
impl ToPyObject for str {
    #[cfg(feature="python27-sys")]
    type ObjectType = PyObject;

    #[cfg(feature="python3-sys")]
    type ObjectType = PyUnicode;

    #[cfg(feature="python27-sys")]
    fn to_py_object(&self, py : Python) -> PyObject {
        if self.is_ascii() {
            PyBytes::new(py, self.as_bytes()).into_object()
        } else {
            PyUnicode::new(py, self).into_object()
        }
    }

    #[cfg(feature="python3-sys")]
    #[inline]
    fn to_py_object(&self, py : Python) -> PyUnicode {
        PyUnicode::new(py, self)
    }
}

/// Converts rust `String` to Python object:
/// ASCII-only strings are converted to Python `str` objects;
/// other strings are converted to Python `unicode` objects.
///
/// Note that `str::ObjectType` differs based on Python version:
/// In Python 2.7, it is `PyObject` (`object` is the common base class of `str` and `unicode`).
/// In Python 3.x, it is `PyUnicode`.
impl ToPyObject for String {
    type ObjectType = <str as ToPyObject>::ObjectType;

    #[inline]
    fn to_py_object(&self, py: Python) -> Self::ObjectType {
        <str as ToPyObject>::to_py_object(self, py)
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
/// In Python 2.7, `str` is expected to be UTF-8 encoded.
extract!(obj to String; py => {
    PyString::extract(py, obj).map(|s| s.into_owned())
});

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
/// In Python 2.7, `str` is expected to be UTF-8 encoded.
extract!(obj to Cow<'prepared, str>; py => {
    PyString::extract(py, obj)
});

enum PreparedString {
    Extracted(String),
    BorrowFrom(PyObject)
}

impl <'prepared> ExtractPyObject<'prepared> for &'prepared str {
    type Prepared = PreparedString;

    fn prepare_extract(py: Python, obj: &PyObject) -> PyResult<Self::Prepared> {
        match try!(PyString::extract(py, obj)) {
            Cow::Owned(s) => Ok(PreparedString::Extracted(s)),
            Cow::Borrowed(_) => Ok(PreparedString::BorrowFrom(obj.clone_ref(py)))
        }
    }

    fn extract(py: Python, prepared: &'prepared PreparedString) -> PyResult<Self> {
        match *prepared {
            PreparedString::Extracted(ref s) => Ok(s),
            PreparedString::BorrowFrom(ref obj) => {
                match try!(PyString::extract(py, obj)) {
                    Cow::Owned(_) => panic!("Failed to borrow from python object"),
                    Cow::Borrowed(s) => Ok(s)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use python::{Python, PythonObject};
    use conversion::{ToPyObject, ExtractPyObject};

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
        let prepared = <&str>::prepare_extract(py, &py_string).unwrap();
        assert_eq!(s, <&str>::extract(py, &prepared).unwrap());
    }
}

