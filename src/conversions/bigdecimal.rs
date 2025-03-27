#![cfg(feature = "bigdecimal")]
//! Conversions to and from [bigdecimal](https://docs.rs/bigdecimal)'s [`BigDecimal`] type.
//!
//! This is useful for converting Python's decimal.Decimal into and from a native Rust type.
//!
//! # Setup
//!
//! To use this feature, add to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"bigdecimal\"] }")]
//! bigdecimal = "4.0"
//! ```
//!
//! Note that you must use a compatible version of bigdecimal and PyO3.
//! The required bigdecimal version may vary based on the version of PyO3.
//!
//! # Example
//!
//! Rust code to create a function that adds one to a BigDecimal
//!
//! ```rust
//! use bigdecimal::BigDecimal;
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn add_one(d: BigDecimal) -> BigDecimal {
//!     d + 1
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(add_one, m)?)?;
//!     Ok(())
//! }
//! ```
//!
//! Python code that validates the functionality
//!
//!
//! ```python
//! from my_module import add_one
//! from decimal import Decimal
//!
//! d = Decimal("2")
//! value = add_one(d)
//!
//! assert d + 1 == value
//! ```

use std::str::FromStr;

use crate::{
    exceptions::PyValueError,
    sync::GILOnceCell,
    types::{PyAnyMethods, PyStringMethods, PyType},
    Bound, FromPyObject, IntoPyObject, Py, PyAny, PyErr, PyResult, Python,
};
use bigdecimal::{BigDecimal, BigDecimalRef};

static DECIMAL_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();

fn get_decimal_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    DECIMAL_CLS.import(py, "decimal", "Decimal")
}

impl FromPyObject<'_> for BigDecimal {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let py_str = &obj.str()?;
        let rs_str = &py_str.to_cow()?;
        BigDecimal::from_str(rs_str).map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

impl<'py> IntoPyObject<'py> for BigDecimal {
    type Target = PyAny;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let cls = get_decimal_cls(py)?;
        cls.call1((self.to_string(),))
    }
}

impl<'py> IntoPyObject<'py> for BigDecimalRef<'_> {
    type Target = PyAny;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let cls = get_decimal_cls(py)?;
        cls.call1((self.to_string(),))
    }
}
