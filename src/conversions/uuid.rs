#![cfg(feature = "uuid")]

//! Conversions to and from [uuid](https://docs.rs/uuid/latest/uuid/)'s [`Uuid`] type.
//!
//! This is useful for converting Python's uuid.UUID into and from a native Rust type.
//!
//! # Setup
//!
//! To use this feature, add to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"uuid\"] }")]
//! uuid = "1.11.0"
//! ```
//!
//! Note that you must use a compatible version of uuid and PyO3.
//! The required uuid version may vary based on the version of PyO3.
//!
//! # Example
//!
//! Rust code to create a function that parses a UUID string and returns it as a `Uuid`:
//!
//! ```rust
//! use pyo3::prelude::*;
//! use pyo3::exceptions::PyValueError;
//! use uuid::Uuid;
//!
//! #[pyfunction]
//! fn parse_uuid(s: &str) -> PyResult<Uuid> {
//!     Uuid::parse_str(s).map_err(|e| PyValueError::new_err(e.to_string()))
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(parse_uuid, m)?)?;
//!     Ok(())
//! }
//! ```
//!
//! Python code that validates the functionality
//!
//!
//! ```python
//! from my_module import parse_uuid
//! import uuid
//!
//! py_uuid = uuid.uuid4()
//! rust_uuid = parse_uuid(str(py_uuid))
//!
//! assert py_uuid == rust_uuid
//! ```
use uuid::Uuid;

use crate::conversion::IntoPyObject;
use crate::exceptions::{PyTypeError, PyValueError};
use crate::instance::Bound;
use crate::sync::GILOnceCell;
use crate::types::any::PyAnyMethods;
use crate::types::{
    PyBytes, PyBytesMethods, PyInt, PyStringMethods, PyType,
};
use crate::{FromPyObject, Py, PyAny, PyErr, PyObject, PyResult, Python};
#[allow(deprecated)]
use crate::{IntoPy, ToPyObject};

static UUID_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();

fn get_uuid_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    UUID_CLS.import(py, "uuid", "UUID")
}

impl FromPyObject<'_> for Uuid {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py = obj.py();

        if let Ok(uuid_cls) = get_uuid_cls(py) {
            if obj.is_exact_instance(uuid_cls) {
                let uuid_int: u128 = obj.getattr("int")?.extract()?;
                return Ok(Uuid::from_u128(uuid_int.to_le()));
            }
        }

        if obj.is_instance_of::<PyBytes>() {
            let bytes = if let Ok(py_bytes) = obj.downcast::<PyBytes>() {
                py_bytes.as_bytes()
            } else {
                return Err(PyTypeError::new_err(
                    "Expected bytes for UUID extraction.",
                ));
            };

            return Uuid::from_slice(bytes)
                .map_err(|_| PyValueError::new_err("The given bytes value is not a valid UUID."));
        }

        if obj.is_instance_of::<PyInt>() {
            let uuid_int: u128 = obj.extract().map_err(|_| {
                PyTypeError::new_err(
                    "Expected integer for UUID extraction but got an incompatible type.",
                )
            })?;
            return Ok(Uuid::from_u128(uuid_int));
        }

        let py_str = &obj.str()?;
        let rs_str = &py_str.to_cow()?;
        Uuid::parse_str(rs_str)
            .map_err(|e| PyValueError::new_err(format!("Invalid UUID string: {e}")))
    }
}

#[allow(deprecated)]
impl ToPyObject for Uuid {
    #[inline]
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().unbind()
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for Uuid {
    #[inline]
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.into_pyobject(py).unwrap().unbind()
    }
}

impl<'py> IntoPyObject<'py> for Uuid {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let uuid_cls = get_uuid_cls(py)?;

        Ok(uuid_cls
            .call1((py.None(), py.None(), py.None(), py.None(), self.as_u128()))?
            .into_pyobject(py)?)
    }
}

impl<'py> IntoPyObject<'py> for &Uuid {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::dict::PyDictMethods;
    use crate::types::{PyDict, PyString};
    use std::ffi::CString;
    use uuid::Uuid;

    macro_rules! convert_constants {
        ($name:ident, $rs:expr, $py:literal) => {
            #[test]
            fn $name() -> PyResult<()> {
                Python::with_gil(|py| {
                    let rs_orig = $rs;
                    let rs_uuid = rs_orig.into_pyobject(py).unwrap();
                    let locals = PyDict::new(py);
                    locals.set_item("rs_uuid", &rs_uuid).unwrap();

                    py.run(
                        &CString::new(format!(
                            "import uuid\npy_uuid = uuid.UUID('{}')\nassert py_uuid == rs_uuid",
                            $py
                        ))
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let py_uuid = locals.get_item("py_uuid").unwrap().unwrap();
                    let py_result: Uuid = py_uuid.extract().unwrap();
                    assert_eq!(rs_orig, py_result);

                    Ok(())
                })
            }
        };
    }

    convert_constants!(
        convert_nil,
        Uuid::nil(),
        "00000000-0000-0000-0000-000000000000"
    );
    convert_constants!(
        convert_max,
        Uuid::max(),
        "ffffffff-ffff-ffff-ffff-ffffffffffff"
    );

    convert_constants!(
        convert_uuid_v4,
        Uuid::parse_str("a4f6d1b9-1898-418f-b11d-ecc6fe1e1f00").unwrap(),
        "a4f6d1b9-1898-418f-b11d-ecc6fe1e1f00"
    );

    convert_constants!(
        convert_uuid_v3,
        Uuid::parse_str("6fa459ea-ee8a-3ca4-894e-db77e160355e").unwrap(),
        "6fa459ea-ee8a-3ca4-894e-db77e160355e"
    );

    convert_constants!(
        convert_uuid_v1,
        Uuid::parse_str("a6cc5730-2261-11ee-9c43-2eb5a363657c").unwrap(),
        "a6cc5730-2261-11ee-9c43-2eb5a363657c"
    );

    #[test]
    fn test_uuid_str() {
        Python::with_gil(|py| {
            let s = PyString::new(py, "a6cc5730-2261-11ee-9c43-2eb5a363657c");
            let uuid: Uuid = s.extract().unwrap();
            assert_eq!(
                uuid,
                Uuid::parse_str("a6cc5730-2261-11ee-9c43-2eb5a363657c").unwrap()
            );
        });
    }

    #[test]
    fn test_uuid_bytes() {
        Python::with_gil(|py| {
            let s = PyBytes::new(
                py,
                &[
                    0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5,
                    0xd6, 0xd7, 0xd8,
                ],
            );
            let uuid: Uuid = s.extract().unwrap();
            assert_eq!(
                uuid,
                Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8").unwrap()
            );
        });
    }

    #[test]
    fn test_invalid_uuid_bytes() {
        Python::with_gil(|py| {
            let s = PyBytes::new(
                py,
                &[
                    0xa1, 0xa2, 0xa3, 0xa4, 0xb1, 0xb2, 0xc1, 0xc2, 0xd1, 0xd2, 0xd3, 0xd4, 0xd5,
                    0xd6, 0xd7,
                ],
            );
            let uuid: Result<Uuid, PyErr> = s.extract();
            assert!(uuid.is_err())
        });
    }

    #[test]
    fn test_uuid_int() {
        Python::with_gil(|py| {
            let v = 0xa1a2a3a4b1b2c1c2d1d2d3d4d5d6d7d8u128;
            let obj: Bound<'_, PyInt> = v.into_pyobject(py).unwrap();
            let uuid: Uuid = obj.extract().unwrap();
            assert_eq!(
                uuid,
                Uuid::parse_str("a1a2a3a4-b1b2-c1c2-d1d2-d3d4d5d6d7d8").unwrap()
            );
        });
    }

    #[test]
    fn test_invalid_uuid_int() {
        Python::with_gil(|py| {
            let v = -42;
            let obj: Bound<'_, PyInt> = v.into_pyobject(py).unwrap();
            let uuid: Result<Uuid, PyErr> = obj.extract();
            assert!(uuid.is_err())
        });
    }

    #[test]
    fn test_uuid_incorrect_length() {
        Python::with_gil(|py| {
            let s = PyString::new(py, "123e4567-e89b-12d3-a456-42661417400");
            let uuid: Result<Uuid, PyErr> = s.extract();
            assert!(uuid.is_err())
        });
    }

    #[test]
    fn test_invalid_uuid_string() {
        Python::with_gil(|py| {
            let s = PyString::new(py, "invalid-uuid-str");
            let uuid: Result<Uuid, PyErr> = s.extract();
            assert!(uuid.is_err())
        });
    }
}
