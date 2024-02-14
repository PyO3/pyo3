//! Contains types for working with Python objects that own the underlying data.

use std::{ops::Deref, ptr::NonNull};

use crate::{
    types::{
        any::PyAnyMethods, bytearray::PyByteArrayMethods, bytes::PyBytesMethods,
        string::PyStringMethods, PyByteArray, PyBytes, PyString,
    },
    Bound, DowncastError, FromPyObject, Py, PyAny, PyErr, PyResult,
};

/// A wrapper around `str` where the storage is owned by a Python `bytes` or `str` object.
///
/// This type gives access to the underlying data via a `Deref` implementation.
pub struct PyBackedStr {
    #[allow(dead_code)] // only held so that the storage is not dropped
    storage: Py<PyAny>,
    data: NonNull<[u8]>,
}

impl Deref for PyBackedStr {
    type Target = str;
    fn deref(&self) -> &str {
        // Safety: `data` is known to be immutable utf8 string and owned by self
        unsafe { std::str::from_utf8_unchecked(self.data.as_ref()) }
    }
}

impl TryFrom<Bound<'_, PyString>> for PyBackedStr {
    type Error = PyErr;
    fn try_from(py_string: Bound<'_, PyString>) -> Result<Self, Self::Error> {
        #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
        {
            let s = py_string.to_str()?;
            let data = NonNull::from(s.as_bytes());
            Ok(Self {
                storage: py_string.as_any().to_owned().unbind(),
                data,
            })
        }
        #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
        {
            let bytes = py_string.encode_utf8()?;
            let b = bytes.as_bytes();
            let data = NonNull::from(b);
            let length = b.len();
            Ok(Self {
                storage: bytes.into_any().unbind(),
                data,
            })
        }
    }
}

impl FromPyObject<'_> for PyBackedStr {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py_string = obj.downcast::<PyString>()?.to_owned();
        Self::try_from(py_string)
    }
}

/// A wrapper around `[u8]` where the storage is either owned by a Python `bytes` object, or a Rust `Box<[u8]>`.
///
/// This type gives access to the underlying data via a `Deref` implementation.
pub struct PyBackedBytes {
    #[allow(dead_code)] // only held so that the storage is not dropped
    storage: PyBackedBytesStorage,
    data: NonNull<[u8]>,
}

#[allow(dead_code)]
enum PyBackedBytesStorage {
    Python(Py<PyBytes>),
    Rust(Box<[u8]>),
}

impl Deref for PyBackedBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        // Safety: `data` is known to be immutable and owned by self
        unsafe { self.data.as_ref() }
    }
}

impl From<Bound<'_, PyBytes>> for PyBackedBytes {
    fn from(py_bytes: Bound<'_, PyBytes>) -> Self {
        let b = py_bytes.as_bytes();
        let data = NonNull::from(b);
        Self {
            storage: PyBackedBytesStorage::Python(py_bytes.to_owned().unbind()),
            data,
        }
    }
}

impl From<Bound<'_, PyByteArray>> for PyBackedBytes {
    fn from(py_bytearray: Bound<'_, PyByteArray>) -> Self {
        let s = py_bytearray.to_vec().into_boxed_slice();
        let data = NonNull::from(s.as_ref());
        Self {
            storage: PyBackedBytesStorage::Rust(s),
            data,
        }
    }
}

impl FromPyObject<'_> for PyBackedBytes {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(bytes) = obj.downcast::<PyBytes>() {
            Ok(Self::from(bytes.to_owned()))
        } else if let Ok(bytearray) = obj.downcast::<PyByteArray>() {
            Ok(Self::from(bytearray.to_owned()))
        } else {
            Err(DowncastError::new(obj, "`bytes` or `bytearray`").into())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Python;

    #[test]
    fn py_backed_str_empty() {
        Python::with_gil(|py| {
            let s = PyString::new_bound(py, "");
            let py_backed_str = s.extract::<PyBackedStr>().unwrap();
            assert_eq!(&*py_backed_str, "");
        });
    }

    #[test]
    fn py_backed_str() {
        Python::with_gil(|py| {
            let s = PyString::new_bound(py, "hello");
            let py_backed_str = s.extract::<PyBackedStr>().unwrap();
            assert_eq!(&*py_backed_str, "hello");
        });
    }

    #[test]
    fn py_backed_str_try_from() {
        Python::with_gil(|py| {
            let s = PyString::new_bound(py, "hello");
            let py_backed_str = PyBackedStr::try_from(s).unwrap();
            assert_eq!(&*py_backed_str, "hello");
        });
    }

    #[test]
    fn py_backed_bytes_empty() {
        Python::with_gil(|py| {
            let b = PyBytes::new_bound(py, &[]);
            let py_backed_bytes = b.extract::<PyBackedBytes>().unwrap();
            assert_eq!(&*py_backed_bytes, &[]);
        });
    }

    #[test]
    fn py_backed_bytes() {
        Python::with_gil(|py| {
            let b = PyBytes::new_bound(py, b"abcde");
            let py_backed_bytes = b.extract::<PyBackedBytes>().unwrap();
            assert_eq!(&*py_backed_bytes, b"abcde");
        });
    }

    #[test]
    fn py_backed_bytes_from_bytes() {
        Python::with_gil(|py| {
            let b = PyBytes::new_bound(py, b"abcde");
            let py_backed_bytes = PyBackedBytes::from(b);
            assert_eq!(&*py_backed_bytes, b"abcde");
        });
    }

    #[test]
    fn py_backed_bytes_from_bytearray() {
        Python::with_gil(|py| {
            let b = PyByteArray::new_bound(py, b"abcde");
            let py_backed_bytes = PyBackedBytes::from(b);
            assert_eq!(&*py_backed_bytes, b"abcde");
        });
    }
}
