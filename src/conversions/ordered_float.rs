#![cfg(feature = "ordered-float")]
//! Conversions to and from [ordered-float](https://docs.rs/ordered-float) types.
//! [`NotNan`]`<`[`f32`]`>` and [`NotNan`]`<`[`f64`]`>`.
//! [`OrderedFloat`]`<`[`f32`]`>` and [`OrderedFloat`]`<`[`f64`]`>`.
//!
//! This is useful for converting between Python's float into and from a native Rust type.
//!
//! Take care when comparing sorted collections of float types between Python and Rust.
//! They will likely differ due to the ambiguous sort order of NaNs in Python.
//
//!
//! To use this feature, add to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"ordered-float\"] }")]
//! ordered-float = "5.0.0"
//! ```
//!
//! # Example
//!
//! Rust code to create functions that add ordered floats:
//!
//! ```rust,no_run
//! use ordered_float::{NotNan, OrderedFloat};
//! use pyo3::prelude::*;
//!
//! #[pyfunction]
//! fn add_not_nans(a: NotNan<f64>, b: NotNan<f64>) -> NotNan<f64> {
//!     a + b
//! }
//!
//! #[pyfunction]
//! fn add_ordered_floats(a: OrderedFloat<f64>, b: OrderedFloat<f64>) -> OrderedFloat<f64> {
//!     a + b
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(add_not_nans, m)?)?;
//!     m.add_function(wrap_pyfunction!(add_ordered_floats, m)?)?;
//!     Ok(())
//! }
//! ```
//!
//! Python code that validates the functionality:
//! ```python
//! from my_module import add_not_nans, add_ordered_floats
//!
//! assert add_not_nans(1.0,2.0) == 3.0
//! assert add_ordered_floats(1.0,2.0) == 3.0
//! ```

use crate::conversion::IntoPyObject;
use crate::exceptions::PyValueError;
use crate::types::{any::PyAnyMethods, PyFloat};
use crate::{Bound, FromPyObject, PyAny, PyResult, Python};
use ordered_float::{NotNan, OrderedFloat};
use std::convert::Infallible;

macro_rules! float_conversions {
    ($wrapper:ident, $float_type:ty, $constructor:expr) => {
        impl FromPyObject<'_> for $wrapper<$float_type> {
            fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
                let val: $float_type = obj.extract()?;
                $constructor(val)
            }
        }

        impl<'py> IntoPyObject<'py> for $wrapper<$float_type> {
            type Target = PyFloat;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                self.into_inner().into_pyobject(py)
            }
        }

        impl<'py> IntoPyObject<'py> for &$wrapper<$float_type> {
            type Target = PyFloat;
            type Output = Bound<'py, Self::Target>;
            type Error = Infallible;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (*self).into_pyobject(py)
            }
        }
    };
}
float_conversions!(OrderedFloat, f32, |val| Ok(OrderedFloat(val)));
float_conversions!(OrderedFloat, f64, |val| Ok(OrderedFloat(val)));
float_conversions!(NotNan, f32, |val| NotNan::new(val)
    .map_err(|e| PyValueError::new_err(e.to_string())));
float_conversions!(NotNan, f64, |val| NotNan::new(val)
    .map_err(|e| PyValueError::new_err(e.to_string())));

#[cfg(test)]
mod test_ordered_float {
    use super::*;
    use crate::ffi::c_str;
    use crate::py_run;

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    macro_rules! float_roundtrip_tests {
        ($wrapper:ident, $float_type:ty, $constructor:expr, $standard_test:ident, $wasm_test:ident, $infinity_test:ident, $zero_test:ident) => {
            #[cfg(not(target_arch = "wasm32"))]
            proptest! {
            #[test]
            fn $standard_test(inner_f: $float_type) {
                let f = $constructor(inner_f);

                Python::attach(|py| {
                    let f_py: Bound<'_, PyFloat>  = f.into_pyobject(py).unwrap();

                    py_run!(
                        py,
                        f_py,
                        &format!(
                            "import math\nassert math.isclose(f_py, {})",
                             inner_f as f64 // Always interpret the literal rs float value as f64
                                            // so that it's comparable with the python float
                        )
                    );

                    let roundtripped_f: $wrapper<$float_type> = f_py.extract().unwrap();

                    assert_eq!(f, roundtripped_f);
                })
            }
            }

            #[cfg(target_arch = "wasm32")]
            #[test]
            fn $wasm_test() {
                let inner_f = 10.0;
                let f = $constructor(inner_f);

                Python::attach(|py| {
                    let f_py: Bound<'_, PyFloat>  = f.into_pyobject(py).unwrap();

                    py_run!(
                        py,
                        f_py,
                        &format!(
                            "import math\nassert math.isclose(f_py, {})",
                            inner_f as f64 // Always interpret the literal rs float value as f64
                                           // so that it's comparable with the python float
                        )
                    );

                    let roundtripped_f: $wrapper<$float_type> = f_py.extract().unwrap();

                    assert_eq!(f, roundtripped_f);
                })
            }

            #[test]
            fn $infinity_test() {
                let inner_pinf = <$float_type>::INFINITY;
                let pinf = $constructor(inner_pinf);

                let inner_ninf = <$float_type>::NEG_INFINITY;
                let ninf = $constructor(inner_ninf);

                Python::attach(|py| {
                    let pinf_py: Bound<'_, PyFloat>  = pinf.into_pyobject(py).unwrap();
                    let ninf_py: Bound<'_, PyFloat>  = ninf.into_pyobject(py).unwrap();

                    py_run!(
                        py,
                        pinf_py ninf_py,
                        "\
                        assert pinf_py == float('inf')\n\
                        assert ninf_py == float('-inf')"
                    );

                    let roundtripped_pinf: $wrapper<$float_type> = pinf_py.extract().unwrap();
                    let roundtripped_ninf: $wrapper<$float_type> = ninf_py.extract().unwrap();

                    assert_eq!(pinf, roundtripped_pinf);
                    assert_eq!(ninf, roundtripped_ninf);
                })
            }

            #[test]
            fn $zero_test() {
                let inner_pzero: $float_type = 0.0;
                let pzero = $constructor(inner_pzero);

                let inner_nzero: $float_type = -0.0;
                let nzero = $constructor(inner_nzero);

                Python::attach(|py| {
                    let pzero_py: Bound<'_, PyFloat>  = pzero.into_pyobject(py).unwrap();
                    let nzero_py: Bound<'_, PyFloat>  = nzero.into_pyobject(py).unwrap();

                    // This python script verifies that the values are 0.0 in magnitude
                    // and that the signs are correct(+0.0 vs -0.0)
                    py_run!(
                        py,
                        pzero_py nzero_py,
                        "\
                        import math\n\
                        assert pzero_py == 0.0\n\
                        assert math.copysign(1.0, pzero_py) > 0.0\n\
                        assert nzero_py == 0.0\n\
                        assert math.copysign(1.0, nzero_py) < 0.0"
                    );

                    let roundtripped_pzero: $wrapper<$float_type> = pzero_py.extract().unwrap();
                    let roundtripped_nzero: $wrapper<$float_type> = nzero_py.extract().unwrap();

                    assert_eq!(pzero, roundtripped_pzero);
                    assert_eq!(roundtripped_pzero.signum(), 1.0);
                    assert_eq!(nzero, roundtripped_nzero);
                    assert_eq!(roundtripped_nzero.signum(), -1.0);
                })
            }
        };
    }
    float_roundtrip_tests!(
        OrderedFloat,
        f32,
        OrderedFloat,
        ordered_float_f32_standard,
        ordered_float_f32_wasm,
        ordered_float_f32_infinity,
        ordered_float_f32_zero
    );
    float_roundtrip_tests!(
        OrderedFloat,
        f64,
        OrderedFloat,
        ordered_float_f64_standard,
        ordered_float_f64_wasm,
        ordered_float_f64_infinity,
        ordered_float_f64_zero
    );
    float_roundtrip_tests!(
        NotNan,
        f32,
        |val| NotNan::new(val).unwrap(),
        not_nan_f32_standard,
        not_nan_f32_wasm,
        not_nan_f32_infinity,
        not_nan_f32_zero
    );
    float_roundtrip_tests!(
        NotNan,
        f64,
        |val| NotNan::new(val).unwrap(),
        not_nan_f64_standard,
        not_nan_f64_wasm,
        not_nan_f64_infinity,
        not_nan_f64_zero
    );

    macro_rules! ordered_float_pynan_tests {
        ($test_name:ident, $float_type:ty) => {
            #[test]
            fn $test_name() {
                let inner_nan: $float_type = <$float_type>::NAN;
                let nan = OrderedFloat(inner_nan);

                Python::attach(|py| {
                    let nan_py: Bound<'_, PyFloat> = nan.into_pyobject(py).unwrap();

                    py_run!(
                        py,
                        nan_py,
                        "\
                        import math\n\
                        assert math.isnan(nan_py)"
                    );

                    let roundtripped_nan: OrderedFloat<$float_type> = nan_py.extract().unwrap();

                    assert_eq!(nan, roundtripped_nan);
                })
            }
        };
    }
    ordered_float_pynan_tests!(test_ordered_float_pynan_f32, f32);
    ordered_float_pynan_tests!(test_ordered_float_pynan_f64, f64);

    macro_rules! not_nan_pynan_tests {
        ($test_name:ident, $float_type:ty) => {
            #[test]
            fn $test_name() {
                Python::attach(|py| {
                    let nan_py = py.eval(c_str!("float('nan')"), None, None).unwrap();

                    let nan_rs: PyResult<NotNan<$float_type>> = nan_py.extract();

                    assert!(nan_rs.is_err());
                })
            }
        };
    }
    not_nan_pynan_tests!(test_not_nan_pynan_f32, f32);
    not_nan_pynan_tests!(test_not_nan_pynan_f64, f64);

    macro_rules! py64_rs32 {
        ($test_name:ident, $wrapper:ident, $float_type:ty) => {
            #[test]
            fn $test_name() {
                Python::attach(|py| {
                    let py_64 = py
                        .import("sys")
                        .unwrap()
                        .getattr("float_info")
                        .unwrap()
                        .getattr("max")
                        .unwrap();
                    let rs_32 = py_64.extract::<$wrapper<f32>>().unwrap();
                    // The python f64 is not representable in a rust f32
                    assert!(rs_32.is_infinite());
                })
            }
        };
    }
    py64_rs32!(ordered_float_f32, OrderedFloat, f32);
    py64_rs32!(ordered_float_f64, OrderedFloat, f64);
    py64_rs32!(not_nan_f32, NotNan, f32);
    py64_rs32!(not_nan_f64, NotNan, f64);
}
