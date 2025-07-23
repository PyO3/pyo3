#![cfg(feature = "bytes")]

//! Conversions to and from [bytes](https://docs.rs/bytes/latest/bytes/)'s [`Bytes`].
//!
//! This is useful for efficiently converting Python's `bytes` types efficiently.
//! While `bytes` will be directly borrowed, converting from `bytearray` will result in a copy.
//!
//! When converting `Bytes` back into Python, this will do a copy, just like `&[u8]` and `Vec<u8>`.
//!
//! # When to use `Bytes`
//!
//! Unless you specifically need [`Bytes`] for ref-counted ownership and sharing,
//! you may find that using `&[u8]`, `Vec<u8>`, [`Bound<PyBytes>`], or [`PyBackedBytes`]
//! is simpler for most use cases.
//!
//! # Setup
//!
//! To use this feature, add in your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! bytes = "1.10"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"bytes\"] }")]
//!
//! Note that you must use compatible versions of bytes and PyO3.
//!
//! # Example
//!
//! Rust code to create functions which return `Bytes` or take `Bytes` as arguments:
//!
//! ```rust,no_run
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn get_message_bytes() -> Bytes {
//!     Bytes::from_static(b"Hello Python!")
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
use bytes::Bytes;

use crate::conversion::IntoPyObject;
use crate::instance::Bound;
use crate::pybacked::PyBackedBytes;
use crate::types::any::PyAnyMethods;
use crate::types::PyBytes;
use crate::{FromPyObject, PyAny, PyErr, PyResult, Python};

impl FromPyObject<'_> for Bytes {
    fn extract_bound(ob: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Bytes::from_owner(ob.extract::<PyBackedBytes>()?))
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

impl<'py> IntoPyObject<'py> for &Bytes {
    type Target = PyBytes;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(PyBytes::new(py, self))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PyByteArray, PyByteArrayMethods, PyBytes};
    use crate::Python;

    #[test]
    fn test_bytes() {
        Python::attach(|py| {
            let py_bytes = PyBytes::new(py, b"foobar");
            let bytes: Bytes = py_bytes.extract().unwrap();
            assert_eq!(&*bytes, b"foobar");

            let bytes = Bytes::from_static(b"foobar").into_pyobject(py).unwrap();
            assert!(bytes.is_instance_of::<PyBytes>());
        });
    }

    #[test]
    fn test_bytearray() {
        Python::attach(|py| {
            let py_bytearray = PyByteArray::new(py, b"foobar");
            let bytes: Bytes = py_bytearray.extract().unwrap();
            assert_eq!(&*bytes, b"foobar");

            // Editing the bytearray should not change extracted Bytes
            unsafe { py_bytearray.as_bytes_mut()[0] = b'x' };
            assert_eq!(&bytes, "foobar");
            assert_eq!(&py_bytearray.extract::<Vec<u8>>().unwrap(), b"xoobar");
        });
    }
}
