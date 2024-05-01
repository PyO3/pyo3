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
//! ```rust
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

use crate::ffi;
use crate::sync::GILOnceCell;
use crate::types::any::PyAnyMethods;
use crate::types::PyLong;
use crate::types::PyType;
use crate::{Bound, FromPyObject, IntoPy, Py, PyAny, PyObject, PyResult, Python, ToPyObject};
use std::os::raw::c_char;

#[cfg(feature = "num-bigint")]
use num_bigint::BigInt;
use num_rational::Ratio;

static FRACTION_CLS: GILOnceCell<Py<PyType>> = GILOnceCell::new();

fn get_fraction_cls(py: Python<'_>) -> PyResult<&Bound<'_, PyType>> {
    FRACTION_CLS.get_or_try_init_type_ref(py, "fractions", "Fraction")
}

macro_rules! rational_conversion {
    ($int: ty) => {
        impl<'py> FromPyObject<'py> for Ratio<$int> {
            fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
                let py = obj.py();
                let py_numerator_obj = unsafe {
                    ffi::PyObject_GetAttrString(
                        obj.as_ptr(),
                        "numerator\0".as_ptr() as *const c_char,
                    )
                };
                let py_denominator_obj = unsafe {
                    ffi::PyObject_GetAttrString(
                        obj.as_ptr(),
                        "denominator\0".as_ptr() as *const c_char,
                    )
                };
                let numerator_owned: Py<PyLong> =
                    unsafe { Py::from_owned_ptr_or_err(py, ffi::PyNumber_Long(py_numerator_obj))? };
                let denominator_owned: Py<PyLong> = unsafe {
                    Py::from_owned_ptr_or_err(py, ffi::PyNumber_Long(py_denominator_obj))?
                };
                let rs_numerator: $int = numerator_owned.bind(py).extract()?;
                let rs_denominator: $int = denominator_owned.bind(py).extract()?;
                Ok(Ratio::new(rs_numerator, rs_denominator))
            }
        }

        impl ToPyObject for Ratio<$int> {
            fn to_object(&self, py: Python<'_>) -> PyObject {
                let fraction_cls = get_fraction_cls(py).expect("failed to load fractions.Fraction");
                let ret = fraction_cls
                    .call1((self.to_string(),))
                    .expect("failed to call fractions.Fraction(value)");
                ret.to_object(py)
            }
        }
        impl IntoPy<PyObject> for Ratio<$int> {
            fn into_py(self, py: Python<'_>) -> PyObject {
                self.to_object(py)
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
        Python::with_gil(|py| {
            let locals = PyDict::new_bound(py);
            py.run_bound(
                "import fractions\npy_frac = fractions.Fraction(-0.125)",
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
    fn test_fraction_with_fraction_type() {
        Python::with_gil(|py| {
            let locals = PyDict::new_bound(py);
            py.run_bound(
                "import fractions\npy_frac = fractions.Fraction(fractions.Fraction(10))",
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
        Python::with_gil(|py| {
            let locals = PyDict::new_bound(py);
            py.run_bound(
                "import fractions\n\nfrom decimal import Decimal\npy_frac = fractions.Fraction(Decimal(\"1.1\"))",
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
        Python::with_gil(|py| {
            let locals = PyDict::new_bound(py);
            py.run_bound(
                "import fractions\npy_frac = fractions.Fraction(10,5)",
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

    proptest! {
        #[test]
        fn test_int_roundtrip(num in any::<i32>(), den in any::<i32>()) {
            Python::with_gil(|py| {
                let rs_frac = Ratio::new(num, den);
                let py_frac = rs_frac.into_py(py);
                let roundtripped: Ratio<i32> = py_frac.extract(py).unwrap();
                assert_eq!(rs_frac, roundtripped);
            })
        }

        #[test]
        #[cfg(feature = "num-bigint")]
        fn test_big_int_roundtrip(num in any::<f32>()) {
            Python::with_gil(|py| {
                let rs_frac = Ratio::from_float(num).unwrap();
                let py_frac = rs_frac.clone().into_py(py);
                let roundtripped: Ratio<BigInt> = py_frac.extract(py).unwrap();
                assert_eq!(roundtripped, rs_frac);
            })
        }

    }

    #[test]
    fn test_infinity() {
        Python::with_gil(|py| {
            let locals = PyDict::new_bound(py);
            let py_bound = py.run_bound(
                "import fractions\npy_frac = fractions.Fraction(\"Infinity\")",
                None,
                Some(&locals),
            );
            assert!(py_bound.is_err());
        })
    }
}
