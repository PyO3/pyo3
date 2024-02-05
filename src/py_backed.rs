//! This module provides some wrappers around `str` and `[u8]` where the storage is owned by a Python `str` or `bytes` object.
//!
//! This can help avoid copying text or byte data sourced from Python.

use std::ops::Deref;

use crate::{
    types::{
        any::PyAnyMethods, bytearray::PyByteArrayMethods, bytes::PyBytesMethods,
        string::PyStringMethods, PyByteArray, PyBytes, PyString,
    },
    Bound, DowncastError, FromPyObject, Py, PyAny, PyResult,
};

/// A wrapper around `str` where the storage is owned by a Python `bytes` or `str` object.
///
/// This type gives access to the underlying data via a `Deref` implementation.
pub struct PyBackedStr {
    #[allow(dead_code)]
    storage: PyBackedStrStorage,
    data: *const u8,
    length: usize,
}

#[allow(dead_code)]
enum PyBackedStrStorage {
    String(Py<PyString>),
    Bytes(Py<PyBytes>),
}

impl Deref for PyBackedStr {
    type Target = str;
    fn deref(&self) -> &str {
        unsafe {
            // Safety: `data` is a pointer to the start of a valid UTF-8 string of length `length`.
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(self.data, self.length))
        }
    }
}

impl FromPyObject<'_> for PyBackedStr {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py_string = obj.downcast::<PyString>()?;
        #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
        {
            let s = py_string.to_str()?;
            let data = s.as_ptr();
            let length = s.len();
            Ok(Self {
                storage: PyBackedStrStorage::String(py_string.to_owned().unbind()),
                data,
                length,
            })
        }
        #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
        {
            let bytes = py_string.encode_utf8()?;
            let b = bytes.as_bytes();
            let data = b.as_ptr();
            let length = b.len();
            Ok(Self {
                storage: PyBackedStrStorage::Bytes(bytes.unbind()),
                data,
                length,
            })
        }
    }
}

/// A wrapper around `[u8]` where the storage is either owned by a Python `bytes` object, or a Rust `Vec<u8>`.
///
/// This type gives access to the underlying data via a `Deref` implementation.
pub struct PyBackedBytes {
    #[allow(dead_code)] // only held so that the storage is not dropped
    storage: PyBackedBytesStorage,
    data: *const u8,
    length: usize,
}

enum PyBackedBytesStorage {
    Python(Py<PyBytes>),
    Rust(Vec<u8>),
}

impl Deref for PyBackedBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        unsafe {
            // Safety: `data` is a pointer to the start of a buffer of length `length`.
            std::slice::from_raw_parts(self.data, self.length)
        }
    }
}

impl FromPyObject<'_> for PyBackedBytes {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(bytes) = obj.downcast::<PyBytes>() {
            let b = bytes.as_bytes();
            let data = b.as_ptr();
            let len = b.len();
            return Ok(Self {
                storage: PyBackedBytesStorage::Python(bytes.to_owned().unbind()),
                data,
                length: len,
            });
        }

        if let Ok(bytearray) = obj.downcast::<PyByteArray>() {
            let s = bytearray.to_vec();
            let data = s.as_ptr();
            let len = s.len();
            return Ok(Self {
                storage: PyBackedBytesStorage::Rust(s),
                data,
                length: len,
            });
        }

        return Err(DowncastError::new(obj, "`bytes` or `bytearray`").into());
    }
}
