#![cfg(feature = "bigdecimal")]
//! Conversions to and from [bigdecimal](https://docs.rs/bigdecimal)'s [`Decimal`] type.
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
//! rust_decimal = "1.0"
//! ```
//!
//! Note that you must use a compatible version of bigdecimal and PyO3.
//! The required bigdecimal version may vary based on the version of PyO3.
//!
//! # Example
//!
//! Rust code to create a function that adds one to a Decimal
//!
//! ```rust
//! use rust_decimal::Decimal;
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn add_one(d: Decimal) -> Decimal {
//!     d + Decimal::ONE
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

#[allow(deprecated)]
use crate::{IntoPy, ToPyObject};

use crate::{Bound, FromPyObject, IntoPyObject, PyAny, PyErr, PyObject, PyResult, Python};
use bigdecimal::{BigDecimal, BigDecimalRef};

impl FromPyObject<'_> for BigDecimal {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        todo!()
    }
}

impl<'py> IntoPyObject<'py> for BigDecimal {
    type Target = PyAny;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        todo!()
    }
}

#[allow(deprecated)]
impl ToPyObject for BigDecimal {
    fn to_object(&self, py: Python<'_>) -> crate::PyObject {
        todo!()
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for BigDecimal {
    fn into_py(self, py: Python<'_>) -> PyObject {
        todo!()
    }
}

impl FromPyObject<'_> for BigDecimalRef<'_> {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        todo!()
    }
}

impl<'py> IntoPyObject<'py> for BigDecimalRef<'_> {
    type Target = PyAny;

    type Output = Bound<'py, Self::Target>;

    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        todo!()
    }
}

#[allow(deprecated)]
impl ToPyObject for BigDecimalRef<'_> {
    fn to_object(&self, py: Python<'_>) -> crate::PyObject {
        todo!()
    }
}

#[allow(deprecated)]
impl IntoPy<PyObject> for BigDecimalRef<'_> {
    fn into_py(self, py: Python<'_>) -> PyObject {
        todo!()
    }
}
