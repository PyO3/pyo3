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
//! ```rust,no_run
//! use pyo3::prelude::*;
//! use pyo3::exceptions::PyValueError;
//! use uuid::Uuid;
//!
//! /// Parse a UUID from a string.
//! #[pyfunction]
//! fn get_uuid_from_str(s: &str) -> PyResult<Uuid> {
//!     Uuid::parse_str(s).map_err(|e| PyValueError::new_err(e.to_string()))
//! }
//!
//! /// Passing a Python uuid.UUID directly to Rust.
//! #[pyfunction]
//! fn get_uuid(u: Uuid) -> Uuid {
//!     u
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(get_uuid_from_str, m)?)?;
//!     m.add_function(wrap_pyfunction!(get_uuid, m)?)?;
//!     Ok(())
//! }
//! ```
//!
//! Python code that validates the functionality
//!
//!
//! ```python
//! from my_module import get_uuid_from_str, get_uuid
//! import uuid
//!
//! py_uuid = uuid.uuid4()
//!
//! # Convert string to Rust Uuid
//! rust_uuid = get_uuid_from_str(str(py_uuid))
//! assert py_uuid == rust_uuid
//!
//! # Pass Python UUID directly to Rust
//! returned_uuid = get_uuid(py_uuid)
//! assert py_uuid == returned_uuid
//! ```
use uuid::Uuid;

use crate::conversion::IntoPyObject;
use crate::exceptions::PyTypeError;
use crate::instance::Bound;
use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
use crate::types::PyType;
use crate::{intern, FromPyObject, Py, PyAny, PyErr, PyResult, Python};

fn get_uuid_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    static UUID_CLS: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    UUID_CLS.import(py, "uuid", "UUID")
}

impl FromPyObject<'_> for Uuid {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py = obj.py();
        let uuid_cls = get_uuid_cls(py)?;

        if obj.is_instance(uuid_cls)? {
            let uuid_int: u128 = obj.getattr(intern!(py, "int"))?.extract()?;
            Ok(Uuid::from_u128(uuid_int))
        } else {
            Err(PyTypeError::new_err("Expected a `uuid.UUID` instance."))
        }
    }
}

impl<'py> IntoPyObject<'py> for Uuid {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let uuid_cls = get_uuid_cls(py)?;

        uuid_cls.call1((py.None(), py.None(), py.None(), py.None(), self.as_u128()))
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
    use crate::types::PyDict;
    use std::ffi::CString;
    use uuid::Uuid;

    macro_rules! convert_constants {
        ($name:ident, $rs:expr, $py:literal) => {
            #[test]
            fn $name() -> PyResult<()> {
                Python::attach(|py| {
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
}
