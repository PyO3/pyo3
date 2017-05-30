// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use std::{mem, str, char};
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::os::raw::c_char;

use ::{PyPtr, pyptr};
use ffi;
use python::{ToPythonPointer, Python};
use super::{exc, PyObject};
use token::{PyObjectMarker, PythonObjectWithGilToken};
use err::{PyResult, PyErr};
use conversion::{ToPyObject, RefFromPyObject};

/// Represents a Python string.
pub struct PyString<'p>(pyptr<'p>);

pyobject_nativetype!(PyString, PyUnicode_Check, PyUnicode_Type);

/// Represents a Python byte string.
pub struct PyBytes<'p>(pyptr<'p>);

pyobject_nativetype!(PyBytes, PyBytes_Check, PyBytes_Type);


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

impl<'p> PyString<'p> {

    /// Creates a new Python string object.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python<'p>, s: &str) -> PyString<'p> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            PyString(pyptr::from_owned_ptr_or_panic(
                py, ffi::PyUnicode_FromStringAndSize(ptr, len)))
        }
    }

    pub fn from_object(src: &'p PyObject, encoding: &str, errors: &str)
                       -> PyResult<PyString<'p>> {
        unsafe {
            Ok(PyString(
                pyptr::from_owned_ptr_or_err(
                    src.gil(), ffi::PyUnicode_FromEncodedObject(
                        src.as_ptr(),
                        encoding.as_ptr() as *const i8,
                        errors.as_ptr() as *const i8))?))
        }
    }

    /// Gets the python string data in its underlying representation.
    pub fn data(&self) -> PyStringData {
        // TODO: return the original representation instead
        // of forcing the UTF-8 representation to be created.
        unsafe {
            let mut size : ffi::Py_ssize_t = mem::uninitialized();
            let data = ffi::PyUnicode_AsUTF8AndSize(self.0.as_ptr(), &mut size) as *const u8;
            if data.is_null() {
                PyErr::fetch(self.gil()).print(self.gil());
                panic!("PyUnicode_AsUTF8AndSize failed");
            }
            PyStringData::Utf8(std::slice::from_raw_parts(data, size as usize))
        }
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Returns a `UnicodeDecodeError` if the input is not valid unicode
    /// (containing unpaired surrogates).
    pub fn to_string(&self) -> PyResult<Cow<str>> {
        self.data().to_string(self.gil())
    }

    /// Convert the `PyString` into a Rust string.
    ///
    /// Unpaired surrogates invalid UTF-8 sequences are
    /// replaced with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(&self) -> Cow<str> {
        self.data().to_string_lossy()
    }
}

impl<'p> PyBytes<'p> {
    /// Creates a new Python byte string object.
    /// The byte string is initialized by copying the data from the `&[u8]`.
    ///
    /// Panics if out of memory.
    pub fn new(py: Python<'p>, s: &[u8]) -> PyBytes<'p> {
        let ptr = s.as_ptr() as *const c_char;
        let len = s.len() as ffi::Py_ssize_t;
        unsafe {
            PyBytes(pyptr::from_owned_ptr_or_panic(
                py, ffi::PyBytes_FromStringAndSize(ptr, len)))
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

/// Converts Rust `str` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for str {
    #[inline]
    fn to_object(&self, py: Python) -> PyPtr<PyObjectMarker> {
        PyString::new(py, self).to_object(py)
    }
}

/// Converts Rust `Cow<str>` to Python object.
/// See `PyString::new` for details on the conversion.
impl <'a> ToPyObject for Cow<'a, str> {
    #[inline]
    fn to_object(&self, py: Python) -> PyPtr<PyObjectMarker> {
        PyString::new(py, self).to_object(py)
    }
}

/// Converts Rust `String` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object(&self, py: Python) -> PyPtr<PyObjectMarker> {
        PyString::new(py, self).to_object(py)
    }
}

// /// Allows extracting strings from Python objects.
// /// Accepts Python `str` and `unicode` objects.
pyobject_extract!(obj to Cow<'source, str> => {
    try!(obj.cast_as::<PyString>()).to_string()
});


/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
pyobject_extract!(obj to String => {
    let s = try!(obj.cast_as::<PyString>());
    s.to_string().map(Cow::into_owned)
});


impl<'p> RefFromPyObject<'p> for str {
    fn with_extracted<F, R>(obj: &'p PyObject, f: F) -> PyResult<R>
        where F: FnOnce(&str) -> R
    {
        let p = PyObject::from_borrowed_ptr(obj.gil(), obj.as_ptr());
        let s = try!(p.extract::<Cow<str>>());
        Ok(f(&s))
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
        assert_eq!(s, py_string.as_object(py).extract::<String>().unwrap());
    }

    #[test]
    fn test_extract_str() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "Hello Python";
        let py_string = s.to_object(py);
        let mut called = false;
        RefFromPyObject::with_extracted(&py_string.as_object(py),
            |s2: &str| {
                assert_eq!(s, s2);
                called = true;
            }).unwrap();
        assert!(called);
    }
}
