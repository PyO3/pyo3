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
//! fn my_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
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

use crate::exceptions::PyValueError;
use crate::once_cell::GILOnceCell;
use crate::types::PyType;
use crate::{intern, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject};
use rust_decimal::Decimal;
use std::str::FromStr;

impl FromPyObject<'_> for Decimal {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        // use the string representation to not be lossy
        if let Ok(val) = obj.extract() {
            Ok(Decimal::new(val, 0))
        } else {
            Decimal::from_str(obj.str()?.to_str()?)
                .map_err(|e| PyValueError::new_err(e.to_string()))
        }
    }
}

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

#[cfg(test)]
mod test_rust_decimal {
    use super::*;
    use crate::err::PyErr;
    use crate::prelude::*;
    use crate::types::PyDict;
    use rust_decimal::Decimal;

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    macro_rules! convert_constants {
        ($name:ident, $rs:expr, $py:literal) => {
            #[test]
            fn $name() {
                Python::with_gil(|py| {
                    let rs_orig = $rs;
                    let rs_dec = rs_orig.into_py(py);
                    let locals = PyDict::new(py);
                    locals.set_item("rs_dec", &rs_dec).unwrap();
                    // Checks if Rust Decimal -> Python Decimal conversion is correct
                    py.run(
                        &format!(
                            "import decimal\npy_dec = decimal.Decimal({})\nassert py_dec == rs_dec",
                            $py
                        ),
                        None,
                        Some(locals.as_gil_ref()),
                    )
                    .unwrap();
                    // Checks if Python Decimal -> Rust Decimal conversion is correct
                    let py_dec = locals.get_item("py_dec").unwrap();
                    let py_result: Decimal = FromPyObject::extract(py_dec).unwrap();
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
            Python::with_gil(|py| {
                let rs_dec = num.into_py(py);
                let locals = PyDict::new(py);
                locals.set_item("rs_dec", &rs_dec).unwrap();
                py.run(
                    &format!(
                       "import decimal\npy_dec = decimal.Decimal(\"{}\")\nassert py_dec == rs_dec",
                     num),
                None, Some(locals.as_gil_ref())).unwrap();
                let roundtripped: Decimal = rs_dec.extract(py).unwrap();
                assert_eq!(num, roundtripped);
            })
        }

        #[test]
        fn test_integers(num in any::<i64>()) {
            Python::with_gil(|py| {
                let py_num = num.into_py(py);
                let roundtripped: Decimal = py_num.extract(py).unwrap();
                let rs_dec = Decimal::new(num, 0);
                assert_eq!(rs_dec, roundtripped);
            })
        }
    }

    #[test]
    fn test_nan() {
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            py.run(
                "import decimal\npy_dec = decimal.Decimal(\"NaN\")",
                None,
                Some(locals.as_gil_ref()),
            )
            .unwrap();
            let py_dec = locals.get_item("py_dec").unwrap();
            let roundtripped: Result<Decimal, PyErr> = FromPyObject::extract(py_dec);
            assert!(roundtripped.is_err());
        })
    }

    #[test]
    fn test_infinity() {
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            py.run(
                "import decimal\npy_dec = decimal.Decimal(\"Infinity\")",
                None,
                Some(locals.as_gil_ref()),
            )
            .unwrap();
            let py_dec = locals.get_item("py_dec").unwrap();
            let roundtripped: Result<Decimal, PyErr> = FromPyObject::extract(py_dec);
            assert!(roundtripped.is_err());
        })
    }
}
