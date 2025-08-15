#![cfg(feature = "num-rational")]
//! Conversions to and from [num-rational](https://docs.rs/num-rational) types.
//!
//! This is useful for converting between Python's [fractions.Fraction](https://docs.python.org/3/library/fractions.html) into and from a native Rust
//! type.
//!
//!
//! To use this feature, add to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"num-rational\"] }")]
//! num-rational = "0.4.1"
//! ```
//!
//! # Example
//!
//! Rust code to create a function that adds five to a fraction:
//!
//! ```rust,no_run
//! use num_rational::Ratio;
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn add_five_to_fraction(fraction: Ratio<i32>) -> Ratio<i32> {
//!     fraction + Ratio::new(5, 1)
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(add_five_to_fraction, m)?)?;
//!     Ok(())
//! }
//! ```
//!
//! Python code that validates the functionality:
//! ```python
//! from my_module import add_five_to_fraction
//! from fractions import Fraction
//!
//! fraction = Fraction(2,1)
//! fraction_plus_five = add_five_to_fraction(f)
//! assert fraction + 5 == fraction_plus_five
//! ```

use crate::conversion::IntoPyObject;
use crate::ffi;
use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
use crate::types::PyType;
use crate::{Bound, FromPyObject, Py, PyAny, PyErr, PyResult, Python};

#[cfg(feature = "num-bigint")]
use num_bigint::BigInt;
use num_rational::Ratio;

static FRACTION_CLS: PyOnceLock<Py<PyType>> = PyOnceLock::new();

fn get_fraction_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    FRACTION_CLS.import(py, "fractions", "Fraction")
}

macro_rules! rational_conversion {
    ($int: ty) => {
        impl<'py> FromPyObject<'py> for Ratio<$int> {
            fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
                let py = obj.py();
                let py_numerator_obj = obj.getattr(crate::intern!(py, "numerator"))?;
                let py_denominator_obj = obj.getattr(crate::intern!(py, "denominator"))?;
                let numerator_owned = unsafe {
                    Bound::from_owned_ptr_or_err(py, ffi::PyNumber_Long(py_numerator_obj.as_ptr()))?
                };
                let denominator_owned = unsafe {
                    Bound::from_owned_ptr_or_err(
                        py,
                        ffi::PyNumber_Long(py_denominator_obj.as_ptr()),
                    )?
                };
                let rs_numerator: $int = numerator_owned.extract()?;
                let rs_denominator: $int = denominator_owned.extract()?;
                Ok(Ratio::new(rs_numerator, rs_denominator))
            }
        }

        impl<'py> IntoPyObject<'py> for Ratio<$int> {
            type Target = PyAny;
            type Output = Bound<'py, Self::Target>;
            type Error = PyErr;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (&self).into_pyobject(py)
            }
        }

        impl<'py> IntoPyObject<'py> for &Ratio<$int> {
            type Target = PyAny;
            type Output = Bound<'py, Self::Target>;
            type Error = PyErr;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                get_fraction_cls(py)?.call1((self.numer().clone(), self.denom().clone()))
            }
        }
    };
}
rational_conversion!(i8);
rational_conversion!(i16);
rational_conversion!(i32);
rational_conversion!(isize);
rational_conversion!(i64);
#[cfg(feature = "num-bigint")]
rational_conversion!(BigInt);
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::dict::PyDictMethods;
    use crate::types::PyDict;

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;
    #[test]
    fn test_negative_fraction() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            py.run(
                ffi::c_str!("import fractions\npy_frac = fractions.Fraction(-0.125)"),
                None,
                Some(&locals),
            )
            .unwrap();
            let py_frac = locals.get_item("py_frac").unwrap().unwrap();
            let roundtripped: Ratio<i32> = py_frac.extract().unwrap();
            let rs_frac = Ratio::new(-1, 8);
            assert_eq!(roundtripped, rs_frac);
        })
    }
    #[test]
    fn test_obj_with_incorrect_atts() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            py.run(
                ffi::c_str!("not_fraction = \"contains_incorrect_atts\""),
                None,
                Some(&locals),
            )
            .unwrap();
            let py_frac = locals.get_item("not_fraction").unwrap().unwrap();
            assert!(py_frac.extract::<Ratio<i32>>().is_err());
        })
    }

    #[test]
    fn test_fraction_with_fraction_type() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            py.run(
                ffi::c_str!(
                    "import fractions\npy_frac = fractions.Fraction(fractions.Fraction(10))"
                ),
                None,
                Some(&locals),
            )
            .unwrap();
            let py_frac = locals.get_item("py_frac").unwrap().unwrap();
            let roundtripped: Ratio<i32> = py_frac.extract().unwrap();
            let rs_frac = Ratio::new(10, 1);
            assert_eq!(roundtripped, rs_frac);
        })
    }

    #[test]
    fn test_fraction_with_decimal() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            py.run(
                ffi::c_str!("import fractions\n\nfrom decimal import Decimal\npy_frac = fractions.Fraction(Decimal(\"1.1\"))"),
                None,
                Some(&locals),
            )
            .unwrap();
            let py_frac = locals.get_item("py_frac").unwrap().unwrap();
            let roundtripped: Ratio<i32> = py_frac.extract().unwrap();
            let rs_frac = Ratio::new(11, 10);
            assert_eq!(roundtripped, rs_frac);
        })
    }

    #[test]
    fn test_fraction_with_num_den() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            py.run(
                ffi::c_str!("import fractions\npy_frac = fractions.Fraction(10,5)"),
                None,
                Some(&locals),
            )
            .unwrap();
            let py_frac = locals.get_item("py_frac").unwrap().unwrap();
            let roundtripped: Ratio<i32> = py_frac.extract().unwrap();
            let rs_frac = Ratio::new(10, 5);
            assert_eq!(roundtripped, rs_frac);
        })
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_int_roundtrip() {
        Python::attach(|py| {
            let rs_frac = Ratio::new(1i32, 2);
            let py_frac = rs_frac.into_pyobject(py).unwrap();
            let roundtripped: Ratio<i32> = py_frac.extract().unwrap();
            assert_eq!(rs_frac, roundtripped);
            // float conversion
        })
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_big_int_roundtrip() {
        Python::attach(|py| {
            let rs_frac = Ratio::from_float(5.5).unwrap();
            let py_frac = rs_frac.clone().into_pyobject(py).unwrap();
            let roundtripped: Ratio<BigInt> = py_frac.extract().unwrap();
            assert_eq!(rs_frac, roundtripped);
        })
    }

    #[cfg(not(target_arch = "wasm32"))]
    proptest! {
        #[test]
        fn test_int_roundtrip(num in any::<i32>(), den in any::<i32>()) {
            Python::attach(|py| {
                let rs_frac = Ratio::new(num, den);
                let py_frac = rs_frac.into_pyobject(py).unwrap();
                let roundtripped: Ratio<i32> = py_frac.extract().unwrap();
                assert_eq!(rs_frac, roundtripped);
            })
        }

        #[test]
        #[cfg(feature = "num-bigint")]
        fn test_big_int_roundtrip(num in any::<f32>()) {
            Python::attach(|py| {
                let rs_frac = Ratio::from_float(num).unwrap();
                let py_frac = rs_frac.clone().into_pyobject(py).unwrap();
                let roundtripped: Ratio<BigInt> = py_frac.extract().unwrap();
                assert_eq!(roundtripped, rs_frac);
            })
        }

    }

    #[test]
    fn test_infinity() {
        Python::attach(|py| {
            let locals = PyDict::new(py);
            let py_bound = py.run(
                ffi::c_str!("import fractions\npy_frac = fractions.Fraction(\"Infinity\")"),
                None,
                Some(&locals),
            );
            assert!(py_bound.is_err());
        })
    }
}
