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
use std::{char, str};
use std::ascii::AsciiExt;
use std::borrow::Cow;
use libc::c_char;
use ffi;
use python::{Python, PythonObject, ToPythonPointer};
use super::{exc, PyObject};
use err::{self, PyResult, PyErr};
use conversion::{FromPyObject, ToPyObject};

pyobject_newtype!(PyBytes, PyBytes_Check, PyBytes_Type);
pyobject_newtype!(PyUnicode, PyUnicode_Check, PyUnicode_Type);

#[cfg(feature="python27-sys")]
pub use PyBytes as PyString;
#[cfg(feature="python3-sys")]
pub use PyUnicode as PyString;

impl <'p> PyBytes<'p> {
    /// Creates a new python byte string object from the &[u8].
    pub fn new(py: Python<'p>, s: &[u8]) -> PyBytes<'p> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            err::cast_from_owned_ptr_or_panic(py,
                ffi::PyBytes_FromStringAndSize(ptr, len))
        }
    }

    /// Gets the python string data as byte slice.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length)
        }
    }

    // In python 2.7, PyBytes serves as PyString, so it should offer the
    // to_str and to_string_lossy functions:
    #[cfg(feature="python27-sys")]
    pub fn to_str(&self) -> Result<&str, str::Utf8Error> {
        str::from_utf8(self.as_slice())
    }

    #[cfg(feature="python27-sys")]
    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.as_slice())
    }
}

impl <'p> PyUnicode<'p> {
    /// Creates a new unicode string object from the Rust string.
    pub fn new(py: Python<'p>, s: &str) -> PyUnicode<'p> {
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
    pub fn as_slice(&self) -> &[ffi::Py_UNICODE] {
        unsafe {
            let buffer = ffi::PyUnicode_AS_UNICODE(self.as_ptr()) as *const _;
            let length = ffi::PyUnicode_GET_SIZE(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length)
        }
    }*/

    pub fn to_string(&self) -> PyResult<'p, Cow<str>> {
        // TODO: use PyUnicode_AsUTF8AndSize if available
        let py = self.python();
        let bytes: PyBytes = unsafe {
            try!(err::result_cast_from_owned_ptr(py, ffi::PyUnicode_AsUTF8String(self.as_ptr())))
        };
        match str::from_utf8(bytes.as_slice()) {
            Ok(s) => Ok(Cow::Owned(s.to_owned())),
            Err(e) => Err(PyErr::new(try!(exc::UnicodeDecodeError::new_utf8(py, bytes.as_slice(), e))))
        }
    }

    pub fn to_string_lossy(&self) -> Cow<str> {
        // TODO: use PyUnicode_AsUTF8AndSize if available
        // TODO: test how this function handles lone surrogates or otherwise invalid code points
        let py = self.python();
        let bytes: PyBytes = unsafe {
            err::result_cast_from_owned_ptr(py, ffi::PyUnicode_AsUTF8String(self.as_ptr()))
                .ok().expect("Error in PyUnicode_AsUTF8String")
        };
        Cow::Owned(String::from_utf8_lossy(bytes.as_slice()).into_owned())
    }
}

// On PyString (i.e. PyBytes in 2.7, PyUnicode otherwise), put static methods
// for extraction as Cow<str>:
impl <'p> PyString<'p> {
    #[cfg(feature="python27-sys")]
    pub fn extract<'a>(o: &'a PyObject<'p>) -> PyResult<'p, Cow<'a, str>> {
        let py = o.python();
        if let Ok(s) = o.cast_as::<PyBytes>() {
            match s.to_str() {
                Ok(s) => Ok(Cow::Borrowed(s)),
                Err(e) => Err(PyErr::new(try!(exc::UnicodeDecodeError::new_utf8(py, s.as_slice(), e))))
            }
        } else if let Ok(u) = o.cast_as::<PyUnicode>() {
            u.to_string()
        } else {
            Err(PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None))
        }
    }

    #[cfg(feature="python27-sys")]
    pub fn extract_lossy<'a>(o: &'a PyObject<'p>) -> PyResult<'p, Cow<'a, str>> {
        let py = o.python();
        if let Ok(s) = o.cast_as::<PyBytes>() {
            Ok(s.to_string_lossy())
        } else if let Ok(u) = o.cast_as::<PyUnicode>() {
            Ok(u.to_string_lossy())
        } else {
            Err(PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None))
        }
    }

    #[cfg(feature="python3-sys")]
    pub fn extract<'a>(o: &'a PyObject<'p>) -> PyResult<'p, Cow<'a, str>> {
        let py = o.python();
        if let Ok(u) = o.cast_as::<PyUnicode>() {
            u.to_string()
        } else {
            Err(PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None))
        }
    }

    #[cfg(feature="python3-sys")]
    pub fn extract_lossy<'a>(o: &'a PyObject<'p>) -> PyResult<'p, Cow<'a, str>> {
        let py = o.python();
        if let Ok(u) = o.cast_as::<PyUnicode>() {
            Ok(u.to_string_lossy())
        } else {
            Err(PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None))
        }
    }
}

// When converting strings to/from python, we need to copy the string data.
// This means we can implement ToPyObject for str, but FromPyObject only for (Cow)String.

/// Converts rust `str` to python object:
/// ASCII-only strings are converted to python `str` objects;
/// other strings are converted to python `unicode` objects.
impl <'p> ToPyObject<'p> for str {
    #[cfg(feature="python27-sys")]
    type ObjectType = PyObject<'p>;

    #[cfg(feature="python3-sys")]
    type ObjectType = PyUnicode<'p>;

    #[cfg(feature="python27-sys")]
    fn to_py_object(&self, py : Python<'p>) -> PyObject<'p> {
        if self.is_ascii() {
            PyBytes::new(py, self.as_bytes()).into_object()
        } else {
            PyUnicode::new(py, self).into_object()
        }
    }

    #[cfg(feature="python3-sys")]
    #[inline]
    fn to_py_object(&self, py : Python<'p>) -> PyUnicode<'p> {
        PyUnicode::new(py, self)
    }
}

/// Converts rust `&str` to python object:
/// ASCII-only strings are converted to python `str` objects;
/// other strings are converted to python `unicode` objects.
impl <'p, 'a> ToPyObject<'p> for &'a str {
    type ObjectType = <str as ToPyObject<'p>>::ObjectType;

    fn to_py_object(&self, py : Python<'p>) -> Self::ObjectType {
        (**self).to_py_object(py)
    }
}

/// Allows extracting strings from python objects.
/// Accepts python `str` and `unicode` objects.
/// In python 2.7, `str` is expected to be UTF-8 encoded.
impl <'p> FromPyObject<'p> for String {
    fn from_py_object(o: &PyObject<'p>) -> PyResult<'p, String> {
        PyString::extract(o).map(|s| s.into_owned())
    }
}

#[test]
fn test_non_bmp() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let s = "\u{1F30F}";
    let py_string = s.to_py_object(py).into_object();
    assert_eq!(s, py_string.extract::<String>().unwrap());
}
