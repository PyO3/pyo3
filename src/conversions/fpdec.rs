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

#[cfg(test)]
mod test_fpdec {
    use super::*;
    use crate::err::PyErr;
    use crate::types::PyDict;
    use fpdec::Decimal;

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
                    // Checks if Rust fpdec -> Python Decimal conversion is correct
                    py.run(
                        &format!(
                            "import decimal\npy_dec = decimal.Decimal({})\nassert py_dec == rs_dec",
                            $py
                        ),
                        None,
                        Some(locals),
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

    convert_constants!(convert_zero, Decimal::from_str("0").unrwap(), "0");
    convert_constants!(convert_one, Decimal::from_str("1").unrwap(), "1");
    convert_constants!(convert_neg_one, Decimal::from_str("-1").unrwap(), "-1");
    convert_constants!(
        convert_one_thousand,
        Decimal::from_str("1000").unrwap(),
        "1000"
    );
    convert_constants!(
        convert_decimal,
        Decimal::from_str("999.999").unrwap(),
        "999.999"
    );
    convert_constants!(
        convert_neg_decimal,
        Decimal::from_str("-999.999").unrwap(),
        "-999.999"
    );

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #[test]
        fn test_roundtrip(
            val in any::<f64>()) {
            let num = Decimal::try_from(x).unwrap();
            Python::with_gil(|py| {
                let rs_dec = num.into_py(py);
                let locals = PyDict::new(py);
                locals.set_item("rs_dec", &rs_dec).unwrap();
                py.run(
                    &format!(
                       "import decimal\npy_dec = decimal.Decimal(\"{}\")\nassert py_dec == rs_dec",
                     num),
                None, Some(locals)).unwrap();
                let roundtripped: Decimal = rs_dec.extract(py).unwrap();
                assert_eq!(num, roundtripped);
            })
        }

        #[test]
        fn test_integers(num in any::<i64>()) {
            Python::with_gil(|py| {
                let py_num = num.into_py(py);
                let roundtripped: Decimal = py_num.extract(py).unwrap();
                let rs_dec = Decimal::try_from(num).unwrap();
                assert_eq!(rs_dec, roundtripped);
            })
        }
    }
}
