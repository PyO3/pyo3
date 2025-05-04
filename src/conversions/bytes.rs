#![cfg(feature = "bytes")]

//! Conversions to and from [bytes](https://docs.rs/bytes/latest/bytes/)'s [`Bytes`] and
//! [`BytesMut`] types.
//!
//! This is useful for efficiently converting Python's `bytes` and `bytearray` types efficiently.
//!
//! # Setup
//!
//! To use this feature, add in your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"bytes\"] }")]
//! bytes = "1.10"
//!
//! Note that you must use compatible versions of bytes and PyO3.
//!
//! # Example
//!
//! Rust code to create functions which return `Bytes` or take `Bytes` as arguements:
//!
//! ```rust,no_run
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn get_message_bytes() -> Bytes {
//!     Bytes::from(b"Hello Python!".to_vec())
//! }
//!
//! #[pyfunction]
//! fn num_bytes(bytes: Bytes) -> usize {
//!     bytes.len()
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(get_message_bytes, m)?)?;
//!     m.add_function(wrap_pyfunction!(num_bytes, m)?)?;
//!     Ok(())
//! }
//! ```
//!
//! Python code that calls these functions:
//!
//! ```python
//! from my_module import get_message_bytes, num_bytes
//!
//! message = get_message_bytes()
//! assert message == b"Hello Python!"
//!
//! size = num_bytes(message)
//! assert size == 13
//! ```
use bytes::{Bytes, BytesMut};

use crate::conversion::IntoPyObject;
use crate::exceptions::PyTypeError;
use crate::instance::Bound;
use crate::types::any::PyAnyMethods;
use crate::types::{PyByteArray, PyByteArrayMethods, PyBytes, PyBytesMethods};
use crate::{FromPyObject, PyAny, PyErr, PyResult, Python};

impl FromPyObject<'_> for Bytes {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        if let Ok(bytes) = ob.downcast::<PyBytes>() {
            Ok(Bytes::from((*bytes).as_bytes().to_vec()))
        } else if let Ok(bytearray) = ob.downcast::<PyByteArray>() {
            Ok(Bytes::from((*bytearray).to_vec()))
        } else {
            Err(PyTypeError::new_err("expected bytes or bytearray"))
        }
    }
}

impl FromPyObject<'_> for BytesMut {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        let bytes = ob.extract::<Bytes>()?;
        Ok(BytesMut::from(bytes))
    }
}

impl<'py> IntoPyObject<'py> for Bytes {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new(py, &self))
    }
}

impl<'py> IntoPyObject<'py> for BytesMut {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new(py, &self))
    }
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};

    use crate::{
        conversion::IntoPyObject,
        ffi,
        types::{any::PyAnyMethods, PyBytes},
        Python,
    };

    #[test]
    fn test_bytes() {
        Python::with_gil(|py| {
            let py_bytes = py.eval(ffi::c_str!("b'foobar'"), None, None).unwrap();
            let bytes: Bytes = py_bytes.extract().unwrap();
            assert_eq!(bytes, Bytes::from(b"foobar".to_vec()));

            let bytes = Bytes::from(b"foobar".to_vec()).into_pyobject(py).unwrap();
            assert!(bytes.is_instance_of::<PyBytes>());
        });
    }

    #[test]
    fn test_bytearray() {
        Python::with_gil(|py| {
            let py_bytearray = py
                .eval(ffi::c_str!("bytearray(b'foobar')"), None, None)
                .unwrap();
            let bytes: Bytes = py_bytearray.extract().unwrap();
            assert_eq!(bytes, Bytes::from(b"foobar".to_vec()));
        });
    }

    #[test]
    fn test_bytes_mut() {
        Python::with_gil(|py| {
            let py_bytearray = py
                .eval(ffi::c_str!("bytearray(b'foobar')"), None, None)
                .unwrap();
            let bytes: BytesMut = py_bytearray.extract().unwrap();
            assert_eq!(bytes, BytesMut::from(&b"foobar"[..]));

            let bytesmut = BytesMut::from(&b"foobar"[..]).into_pyobject(py).unwrap();
            assert!(bytesmut.is_instance_of::<PyBytes>());
        });
    }

    #[test]
    fn test_bytearray_mut() {
        Python::with_gil(|py| {
            let py_bytearray = py
                .eval(ffi::c_str!("bytearray(b'foobar')"), None, None)
                .unwrap();
            let bytes: BytesMut = py_bytearray.extract().unwrap();
            assert_eq!(bytes, BytesMut::from(&b"foobar"[..]));
        });
    }
}
