#![cfg(feature = "rust_decimal")]
//! Conversions to and from [rust_decimal](https://docs.rs/rust_decimal)'s [`Decimal`] type.
//!
//! This is useful for converting Python's decimal.Decimal into and from a native Rust type.
//!
//! # Setup
//!
//! To use this feature, add to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"rust_decimal\"] }")]
//! rust_decimal = "1.0"
//! ```
//!
//! Note that you must use a compatible version of rust_decimal and PyO3.
//! The required rust_decimal version may vary based on the version of PyO3.
//!
//! # Example
//!
//! Rust code to create a function that adds one to a Decimal
//!
//! ```rust,no_run
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

use crate::conversion::IntoPyObject;
use crate::exceptions::PyValueError;
use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
use crate::types::string::PyStringMethods;
use crate::types::PyType;
use crate::{Bound, FromPyObject, Py, PyAny, PyErr, PyResult, Python};
use rust_decimal::Decimal;
use std::str::FromStr;

impl FromPyObject<'_> for Decimal {
    fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        // use the string representation to not be lossy
        if let Ok(val) = obj.extract() {
            Ok(Decimal::new(val, 0))
        } else {
            let py_str = &obj.str()?;
            let rs_str = &py_str.to_cow()?;
            Decimal::from_str(rs_str).or_else(|_| {
                Decimal::from_scientific(rs_str).map_err(|e| PyValueError::new_err(e.to_string()))
            })
        }
    }
}

static DECIMAL_CLS: PyOnceLock<Py<PyType>> = PyOnceLock::new();

fn get_decimal_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    DECIMAL_CLS.import(py, "decimal", "Decimal")
}

impl<'py> IntoPyObject<'py> for Decimal {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        let dec_cls = get_decimal_cls(py)?;
        // now call the constructor with the Rust Decimal string-ified
        // to not be lossy
        dec_cls.call1((self.to_string(),))
    }
}

impl<'py> IntoPyObject<'py> for &Decimal {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    #[inline]
    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        (*self).into_pyobject(py)
    }
}

#[cfg(test)]
mod test_rust_decimal {
    use super::*;
    use crate::types::dict::PyDictMethods;
    use crate::types::PyDict;
    use std::ffi::CString;

    use crate::ffi;
    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    macro_rules! convert_constants {
        ($name:ident, $rs:expr, $py:literal) => {
            #[test]
            fn $name() {
                Python::attach(|py| {
                    let rs_orig = $rs;
                    let rs_dec = rs_orig.into_pyobject(py).unwrap();
                    let locals = PyDict::new(py);
                    locals.set_item("rs_dec", &rs_dec).unwrap();
                    // Checks if Rust Decimal -> Python Decimal conversion is correct
                    py.run(
                        &CString::new(format!(
                            "import decimal\npy_dec = decimal.Decimal({})\nassert py_dec == rs_dec",
                            $py
                        ))
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();
                    // Checks if Python Decimal -> Rust Decimal conversion is correct
                    let py_dec = locals.get_item("py_dec").unwrap().unwrap();
                    let py_result: Decimal = py_dec.extract().unwrap();
                    assert_eq!(rs_orig, py_result);
                })
            }
        };
    }

    convert_constants!(convert_zero, Decimal::ZERO, "0");
    convert_constants!(convert_one, Decimal::ONE, "1");
    convert_constants!(convert_neg_one, Decimal::NEGATIVE_ONE, "-1");
    convert_constants!(convert_two, Decimal::TWO, "2");
    convert_constants!(convert_ten, Decimal::TEN, "10");
    convert_constants!(convert_one_hundred, Decimal::ONE_HUNDRED, "100");
    convert_constants!(convert_one_thousand, Decimal::ONE_THOUSAND, "1000");

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #[test]
        fn test_roundtrip(
            lo in any::<u32>(),
            mid in any::<u32>(),
            high in any::<u32>(),
            negative in any::<bool>(),
            scale in 0..28u32
        ) {
            let num = Decimal::from_parts(lo, mid, high, negative, scale);
            Python::attach(|py| {
                let rs_dec = num.into_pyobject(py).unwrap();
                let locals = PyDict::new(py);
                locals.set_item("rs_dec", &rs_dec).unwrap();
                py.run(
                    &CString::new(format!(
                       "import decimal\npy_dec = decimal.Decimal(\"{num}\")\nassert py_dec == rs_dec")).unwrap(),
                None, Some(&locals)).unwrap();
                let roundtripped: Decimal = rs_dec.extract().unwrap();
                assert_eq!(num, roundtripped);
            })
        }

        #[test]
        fn test_integers(num in any::<i64>()) {
            Python::attach(|py| {
                let py_num = num.into_pyobject(py).unwrap();
                let roundtripped: Decimal = py_num.extract().unwrap();
                let rs_dec = Decimal::new(num, 0);
                assert_eq!(rs_dec, roundtripped);
            })
        }
    }

    #[test]
    fn test_nan() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            py.run(
                ffi::c_str!("import decimal\npy_dec = decimal.Decimal(\"NaN\")"),
                None,
                Some(&locals),
            )
            .unwrap();
            let py_dec = locals.get_item("py_dec").unwrap().unwrap();
            let roundtripped: Result<Decimal, PyErr> = py_dec.extract();
            assert!(roundtripped.is_err());
        })
    }

    #[test]
    fn test_scientific_notation() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            py.run(
                ffi::c_str!("import decimal\npy_dec = decimal.Decimal(\"1e3\")"),
                None,
                Some(&locals),
            )
            .unwrap();
            let py_dec = locals.get_item("py_dec").unwrap().unwrap();
            let roundtripped: Decimal = py_dec.extract().unwrap();
            let rs_dec = Decimal::from_scientific("1e3").unwrap();
            assert_eq!(rs_dec, roundtripped);
        })
    }

    #[test]
    fn test_infinity() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            py.run(
                ffi::c_str!("import decimal\npy_dec = decimal.Decimal(\"Infinity\")"),
                None,
                Some(&locals),
            )
            .unwrap();
            let py_dec = locals.get_item("py_dec").unwrap().unwrap();
            let roundtripped: Result<Decimal, PyErr> = py_dec.extract();
            assert!(roundtripped.is_err());
        })
    }
}
