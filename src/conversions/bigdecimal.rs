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
//! bigdecimal = "0.4"
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

use crate::types::PyTuple;
use crate::{
    exceptions::PyValueError,
    sync::PyOnceLock,
    types::{PyAnyMethods, PyStringMethods, PyType},
    Bound, FromPyObject, IntoPyObject, Py, PyAny, PyErr, PyResult, Python,
};
use bigdecimal::BigDecimal;
use num_bigint::Sign;

fn get_decimal_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    static DECIMAL_CLS: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    DECIMAL_CLS.import(py, "decimal", "Decimal")
}

fn get_invalid_operation_error_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    static INVALID_OPERATION_CLS: PyOnceLock<Py<PyType>> = PyOnceLock::new();
    INVALID_OPERATION_CLS.import(py, "decimal", "InvalidOperation")
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
        let (bigint, scale) = self.into_bigint_and_scale();
        if scale == 0 {
            return cls.call1((bigint,));
        }
        let exponent = scale.checked_neg().ok_or_else(|| {
            get_invalid_operation_error_cls(py)
                .map_or_else(|err| err, |cls| PyErr::from_type(cls.clone(), ()))
        })?;
        let (sign, digits) = bigint.to_radix_be(10);
        let signed = matches!(sign, Sign::Minus).into_pyobject(py)?;
        let digits = PyTuple::new(py, digits)?;

        cls.call1(((signed, digits, exponent),))
    }
}

#[cfg(test)]
mod test_bigdecimal {
    use super::*;
    use crate::types::dict::PyDictMethods;
    use crate::types::PyDict;
    use std::ffi::CString;

    use crate::ffi;
    use bigdecimal::{One, Zero};
    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    macro_rules! convert_constants {
        ($name:ident, $rs:expr, $py:literal) => {
            #[test]
            fn $name() {
                Python::attach(|py| {
                    let rs_orig = $rs;
                    let rs_dec = rs_orig.clone().into_pyobject(py).unwrap();
                    let locals = PyDict::new(py);
                    locals.set_item("rs_dec", &rs_dec).unwrap();
                    // Checks if BigDecimal -> Python Decimal conversion is correct
                    py.run(
                        &CString::new(format!(
                            "import decimal\npy_dec = decimal.Decimal(\"{}\")\nassert py_dec == rs_dec",
                            $py
                        ))
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();
                    // Checks if Python Decimal -> BigDecimal conversion is correct
                    let py_dec = locals.get_item("py_dec").unwrap().unwrap();
                    let py_result: BigDecimal = py_dec.extract().unwrap();
                    assert_eq!(rs_orig, py_result);
                })
            }
        };
    }

    convert_constants!(convert_zero, BigDecimal::zero(), "0");
    convert_constants!(convert_one, BigDecimal::one(), "1");
    convert_constants!(convert_neg_one, -BigDecimal::one(), "-1");
    convert_constants!(convert_two, BigDecimal::from(2), "2");
    convert_constants!(convert_ten, BigDecimal::from_str("10").unwrap(), "10");
    convert_constants!(
        convert_one_hundred_point_one,
        BigDecimal::from_str("100.1").unwrap(),
        "100.1"
    );
    convert_constants!(
        convert_one_thousand,
        BigDecimal::from_str("1000").unwrap(),
        "1000"
    );
    convert_constants!(
        convert_scientific,
        BigDecimal::from_str("1e10").unwrap(),
        "1e10"
    );

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #[test]
        fn test_roundtrip(
            number in 0..28u32
        ) {
            let num = BigDecimal::from(number);
            Python::attach(|py| {
                let rs_dec = num.clone().into_pyobject(py).unwrap();
                let locals = PyDict::new(py);
                locals.set_item("rs_dec", &rs_dec).unwrap();
                py.run(
                    &CString::new(format!(
                       "import decimal\npy_dec = decimal.Decimal(\"{num}\")\nassert py_dec == rs_dec")).unwrap(),
                None, Some(&locals)).unwrap();
                let roundtripped: BigDecimal = rs_dec.extract().unwrap();
                assert_eq!(num, roundtripped);
            })
        }

        #[test]
        fn test_integers(num in any::<i64>()) {
            Python::attach(|py| {
                let py_num = num.into_pyobject(py).unwrap();
                let roundtripped: BigDecimal = py_num.extract().unwrap();
                let rs_dec = BigDecimal::from(num);
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
            let roundtripped: Result<BigDecimal, PyErr> = py_dec.extract();
            assert!(roundtripped.is_err());
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
            let roundtripped: Result<BigDecimal, PyErr> = py_dec.extract();
            assert!(roundtripped.is_err());
        })
    }

    #[test]
    fn test_no_precision_loss() {
        Python::attach(|py| {
            let src = "1e4";
            let expected = get_decimal_cls(py)
                .unwrap()
                .call1((src,))
                .unwrap()
                .call_method0("as_tuple")
                .unwrap();
            let actual = src
                .parse::<BigDecimal>()
                .unwrap()
                .into_pyobject(py)
                .unwrap()
                .call_method0("as_tuple")
                .unwrap();

            assert!(actual.eq(expected).unwrap());
        });
    }
}
