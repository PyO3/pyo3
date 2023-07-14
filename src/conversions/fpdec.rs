#![cfg(feature = "fpdec")]
//! Conversions to and from [fpdec](https://docs.rs/fpdec)'s [`Decimal`] type.
//!
//! This is useful for converting Python's decimal.Decimal into and from a native Rust type (fpdec).
//!
//! # Setup
//!
//! To use this feature, add to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"fpdec\"] }")]
//! fpdec = "0.8.1"
//! ```
//!
//! Note that you must use a compatible version of fpdec and PyO3.
//! The required fpdec version may vary based on the version of PyO3.
//!
//! ```
//!
use crate::exceptions::PyValueError;
use crate::once_cell::GILOnceCell;
use crate::types::PyType;
use crate::{intern, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject};
use fpdec::Decimal;
use std::str::FromStr;

static DECIMAL_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();

fn get_decimal_cls(py: Python<'_>) -> PyResult<&PyType> {
    DECIMAL_CLS
        .get_or_try_init(py, || {
            py.import(intern!(py, "decimal"))?
                .getattr(intern!(py, "Decimal"))?
                .extract()
        })
        .map(|ty| ty.as_ref(py))
}

impl FromPyObject<'_> for Decimal {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        //use the string representation to not be lossy
        if let Ok(val) = obj.extract::<f64>() {
            Decimal::try_from(val).map_err(|e| PyValueError::new_err(e.to_string()))
        } else {
            Decimal::from_str(obj.str()?.to_str()?)
                .map_err(|e| PyValueError::new_err(e.to_string()))
        }
    }
}
impl ToPyObject for Decimal {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        // TODO: handle error gracefully when ToPyObject can error
        // look up the decimal.Decimal
        let dec_cls = get_decimal_cls(py).expect("failed to load decimal.Decimal");

        // now call the constructor with the Rust Decimal string-ified
        // to not be lossy
        let ret = dec_cls
            .call1((self.to_string(),))
            .expect("failed to call decimal.Decimal(value)");

        ret.to_object(py)
    }
}

impl IntoPy<PyObject> for Decimal {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.to_object(py)
    }
}
