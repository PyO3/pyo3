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
use python::{Python, PythonObject, ToPythonPointer};
use super::{exc, PyObject};
use err::{self, PyResult, PyErr};
use conversion::{ExtractPyObject, ToPyObject};

/// Represents a Python byte string.
/// Corresponds to `str` in Python 2, and `bytes` in Python 3.
pub struct PyBytes<'p>(PyObject<'p>);

/// Represents a Python unicode string.
/// Corresponds to `unicode` in Python 2, and `str` in Python 3.
pub struct PyUnicode<'p>(PyObject<'p>);

pyobject_newtype!(PyBytes, PyBytes_Check, PyBytes_Type);
pyobject_newtype!(PyUnicode, PyUnicode_Check, PyUnicode_Type);

#[cfg(feature="python27-sys")]
pub use PyBytes as PyString;
#[cfg(feature="python3-sys")]
pub use PyUnicode as PyString;

impl <'p> PyBytes<'p> {
    /// Creates a new Python byte string object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python<'p>, s: &[u8]) -> PyBytes<'p> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            err::cast_from_owned_ptr_or_panic(py,
                ffi::PyBytes_FromStringAndSize(ptr, len))
        }
    }

    /// Gets the Python string data as byte slice.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            let buffer = ffi::PyBytes_AsString(self.as_ptr()) as *const u8;
            let length = ffi::PyBytes_Size(self.as_ptr()) as usize;
            std::slice::from_raw_parts(buffer, length)
        }
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

    /// Convert the `PyUnicode` into a rust string.
    ///
    /// Returns a `UnicodeDecodeError` if the input contains invalid code points.
    pub fn to_string(&self) -> PyResult<'p, Cow<str>> {
        // TODO: use PyUnicode_AsUTF8AndSize if available
        let py = self.python();
        let bytes: PyBytes = unsafe {
            try!(err::result_cast_from_owned_ptr(py, ffi::PyUnicode_AsUTF8String(self.as_ptr())))
        };
        match str::from_utf8(bytes.as_slice()) {
            Ok(s) => Ok(Cow::Owned(s.to_owned())),
            Err(e) => Err(PyErr::from_instance(try!(exc::UnicodeDecodeError::new_utf8(py, bytes.as_slice(), e))))
        }
    }

    /// Convert the `PyUnicode` into a rust string.
    ///
    /// Any invalid code points are replaced with U+FFFD REPLACEMENT CHARACTER.
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
    pub fn extract<'a>(o: &'a PyObject<'p>) -> PyResult<'p, Cow<'a, str>> {
        let py = o.python();
        if let Ok(s) = o.cast_as::<PyBytes>() {
            s.to_string()
        } else if let Ok(u) = o.cast_as::<PyUnicode>() {
            u.to_string()
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
    pub fn extract<'a>(o: &'a PyObject<'p>) -> PyResult<'p, Cow<'a, str>> {
        let py = o.python();
        if let Ok(u) = o.cast_as::<PyUnicode>() {
            u.to_string()
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
    pub fn extract_lossy<'a>(o: &'a PyObject<'p>) -> PyResult<'p, Cow<'a, str>> {
        let py = o.python();
        if let Ok(u) = o.cast_as::<PyUnicode>() {
            Ok(u.to_string_lossy())
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
    pub fn to_string(&self) -> PyResult<'p, Cow<str>> {
        let py = self.python();
        match str::from_utf8(self.as_slice()) {
            Ok(s) => Ok(Cow::Borrowed(s)),
            Err(e) => Err(PyErr::from_instance(try!(exc::UnicodeDecodeError::new_utf8(py, self.as_slice(), e))))
        }
    }

    /// Convert the `PyString` into a rust string.
    ///
    /// In Python 2.7, `PyString` is a byte string and will be decoded using UTF-8.
    /// In Python 3.x, `PyString` is a unicode string.
    ///
    /// Any invalid UTF-8 sequences are replaced with U+FFFD REPLACEMENT CHARACTER.
    #[cfg(feature="python27-sys")]
    pub fn to_string_lossy(&self) -> Cow<str> {
        String::from_utf8_lossy(self.as_slice())
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

/// Converts rust `String` to Python object:
/// ASCII-only strings are converted to Python `str` objects;
/// other strings are converted to Python `unicode` objects.
///
/// Note that `str::ObjectType` differs based on Python version:
/// In Python 2.7, it is `PyObject` (`object` is the common base class of `str` and `unicode`).
/// In Python 3.x, it is `PyUnicode`.
impl <'p> ToPyObject<'p> for String {
    type ObjectType = <str as ToPyObject<'p>>::ObjectType;

    #[inline]
    fn to_py_object(&self, py: Python<'p>) -> Self::ObjectType {
        <str as ToPyObject>::to_py_object(self, py)
    }
}

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
/// In Python 2.7, `str` is expected to be UTF-8 encoded.
extract!(obj to String => {
    PyString::extract(obj).map(|s| s.into_owned())
});

/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
/// In Python 2.7, `str` is expected to be UTF-8 encoded.
extract!(obj to Cow<'source, str> => {
    PyString::extract(obj)
});

impl <'python, 'source, 'prepared> ExtractPyObject<'python, 'source, 'prepared> for &'prepared str {

    type Prepared = Cow<'source, str>;

    #[inline]
    fn prepare_extract(obj: &'source PyObject<'python>) -> PyResult<'python, Self::Prepared> {
        PyString::extract(obj)
    }

    #[inline]
    fn extract(cow: &'prepared Cow<'source, str>) -> PyResult<'python, Self> {
        Ok(cow)
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
        assert_eq!(s, py_string.extract::<String>().unwrap());
    }

    #[test]
    fn test_extract_str() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "Hello Python";
        let py_string = s.to_py_object(py).into_object();
        let prepared = <&str>::prepare_extract(&py_string).unwrap();
        assert_eq!(s, <&str>::extract(&prepared).unwrap());
    }
}

