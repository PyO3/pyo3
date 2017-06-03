// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std;
use std::{mem, str, char};
use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::os::raw::c_char;

use ffi;
use pointers::PyPtr;
use python::{ToPyPointer, Python};
use super::{exc, PyObject};
use err::{PyResult, PyErr};
use conversion::{ToPyObject, RefFromPyObject};

/// Represents a Python string.
pub struct PyString(PyPtr);

pyobject_convert!(PyString);
pyobject_nativetype!(PyString, PyUnicode_Check, PyUnicode_Type);

/// Represents a Python byte string.
pub struct PyBytes(PyPtr);

pyobject_convert!(PyBytes);
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

impl PyString {

    /// Creates a new Python string object.
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
                       -> PyResult<PyString> {
        unsafe {
            Ok(PyString(
                PyPtr::from_owned_ptr_or_err(
                    py, ffi::PyUnicode_FromEncodedObject(
                        src.as_ptr(),
                        encoding.as_ptr() as *const i8,
                        errors.as_ptr() as *const i8))?))
        }
    }

    /// Gets the python string data in its underlying representation.
    pub fn data(&self, py: Python) -> PyStringData {
        // TODO: return the original representation instead
        // of forcing the UTF-8 representation to be created.
        unsafe {
            let mut size : ffi::Py_ssize_t = mem::uninitialized();
            let data = ffi::PyUnicode_AsUTF8AndSize(self.0.as_ptr(), &mut size) as *const u8;
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
}

/// Converts Rust `str` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for str {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

/// Converts Rust `Cow<str>` to Python object.
/// See `PyString::new` for details on the conversion.
impl<'a> ToPyObject for Cow<'a, str> {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

/// Converts Rust `String` to Python object.
/// See `PyString::new` for details on the conversion.
impl ToPyObject for String {
    #[inline]
    fn to_object(&self, py: Python) -> PyObject {
        PyString::new(py, self).into()
    }
}

// /// Allows extracting strings from Python objects.
// /// Accepts Python `str` and `unicode` objects.
pyobject_extract!(py, obj to Cow<'source, str> => {
    try!(obj.cast_as::<PyString>(py)).to_string(py)
});


/// Allows extracting strings from Python objects.
/// Accepts Python `str` and `unicode` objects.
pyobject_extract!(py, obj to String => {
    let s = try!(obj.cast_as::<PyString>(py));
    s.to_string(py).map(Cow::into_owned)
});


impl<'p> RefFromPyObject<'p> for str {
    fn with_extracted<F, R>(py: Python, obj: &'p PyObject, f: F) -> PyResult<R>
        where F: FnOnce(&str) -> R
    {
        let p = PyObject::from_borrowed_ptr(py, obj.as_ptr());
        let s = try!(p.extract::<Cow<str>>(py));
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
        assert_eq!(s, py_string.extract::<String>(py).unwrap());
    }

    #[test]
    fn test_extract_str() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let s = "Hello Python";
        let py_string = s.to_object(py);
        let mut called = false;
        RefFromPyObject::with_extracted(py, &py_string.into(),
            |s2: &str| {
                assert_eq!(s, s2);
                called = true;
            }).unwrap();
        assert!(called);
    }
}
