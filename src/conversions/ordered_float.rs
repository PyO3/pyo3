#![cfg(feature = "ordered-float")]

use crate::conversion::IntoPyObject;
use crate::exceptions::PyValueError;
use crate::ffi_ptr_ext::FfiPtrExt;
use crate::types::any::PyAnyMethods;
use crate::{ffi, Bound, FromPyObject, PyAny, PyErr, PyResult, Python};
use ordered_float::{NotNan, OrderedFloat};

macro_rules! ordered_float_conversions {
    ($float_type:ty) => {
        impl FromPyObject<'_> for OrderedFloat<$float_type> {
            fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
                let val: $float_type = obj.extract()?;
                Ok(OrderedFloat(val))
            }
        }

        impl<'py> IntoPyObject<'py> for OrderedFloat<$float_type> {
            type Target = PyAny;
            type Output = Bound<'py, Self::Target>;
            type Error = PyErr;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                let float = unsafe {
                    ffi::PyFloat_FromDouble(self.into_inner() as f64)
                        .assume_owned(py)
                        .downcast_into_unchecked()
                };
                Ok(float)
            }
        }

        impl<'py> IntoPyObject<'py> for &OrderedFloat<$float_type> {
            type Target = PyAny;
            type Output = Bound<'py, Self::Target>;
            type Error = PyErr;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (*self).into_pyobject(py)
            }
        }
    };
}
ordered_float_conversions!(f32);
ordered_float_conversions!(f64);

macro_rules! not_nan_conversions {
    ($float_type:ty) => {
        impl FromPyObject<'_> for NotNan<$float_type> {
            fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
                let val: $float_type = obj.extract()?;
                NotNan::new(val).map_err(|e| PyValueError::new_err(e.to_string()))
            }
        }

        impl<'py> IntoPyObject<'py> for NotNan<$float_type> {
            type Target = PyAny;
            type Output = Bound<'py, Self::Target>;
            type Error = PyErr;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                let float = unsafe {
                    ffi::PyFloat_FromDouble(self.into_inner() as f64)
                        .assume_owned(py)
                        .downcast_into_unchecked()
                };
                Ok(float)
            }
        }

        impl<'py> IntoPyObject<'py> for &NotNan<$float_type> {
            type Target = PyAny;
            type Output = Bound<'py, Self::Target>;
            type Error = PyErr;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (*self).into_pyobject(py)
            }
        }
    };
}
not_nan_conversions!(f32);
not_nan_conversions!(f64);

#[cfg(test)]
mod test_ordered_float {
    use super::*;
    use crate::types::dict::PyDictMethods;
    use crate::types::PyDict;
    use std::ffi::CString;

    #[cfg(not(target_arch = "wasm32"))]
    use proptest::prelude::*;

    macro_rules! ordered_float_roundtrip_tests {
        ($standard_test:ident, $wasm_test:ident, $infinity_test:ident, $zero_test:ident, $nan_test:ident, $float_type:ty) => {
            #[cfg(not(target_arch = "wasm32"))]
            proptest! {
            #[test]
            fn $standard_test(inner_f: $float_type) {
                let f = OrderedFloat(inner_f);

                Python::with_gil(|py| {
                    let f_py = f.into_pyobject(py).unwrap();

                    let locals = PyDict::new(py);
                    locals.set_item("f_py", &f_py).unwrap();

                    py.run(
                        &CString::new(format!(
                            "import math\nassert math.isclose(f_py, {})",
                             inner_f as f64 // Always interpret the literal rs float value as f64
                                            // so that it's comparable with the python float
                        ))
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let roundtripped_f: OrderedFloat<$float_type> = f_py.extract().unwrap();

                    assert_eq!(f, roundtripped_f);
                })
            }
            }

            #[cfg(target_arch = "wasm32")]
            fn $wasm_test() {
                let inner_f = 10.0;
                let f = OrderedFloat(inner_f);

                Python::with_gil(|py| {
                    let f_py = f.into_pyobject(py).unwrap();

                    let locals = PyDict::new(py);
                    locals.set_item("f_py", &f_py).unwrap();

                    py.run(
                        &CString::new(format!(
                            "import math\nassert math.isclose(f_py, {})",
                            inner_f as f64 // Always interpret the literal rs float value as f64
                                           // so that it's comparable with the python float
                        ))
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let roundtripped_f: OrderedFloat<$float_type> = f_py.extract().unwrap();

                    assert_eq!(f, roundtripped_f);
                })
            }

            #[test]
            fn $infinity_test() {
                let inner_pinf = <$float_type>::INFINITY;
                let pinf = OrderedFloat(inner_pinf);

                let inner_ninf = <$float_type>::NEG_INFINITY;
                let ninf = OrderedFloat(inner_ninf);

                Python::with_gil(|py| {
                    let pinf_py = pinf.into_pyobject(py).unwrap();
                    let ninf_py = ninf.into_pyobject(py).unwrap();

                    let locals = PyDict::new(py);
                    locals.set_item("pinf_py", &pinf_py).unwrap();
                    locals.set_item("ninf_py", &ninf_py).unwrap();

                    py.run(
                        &CString::new(
                            "\
                            assert pinf_py == float('inf')\n\
                            assert ninf_py == float('-inf')",
                        )
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let roundtripped_pinf: OrderedFloat<$float_type> = pinf_py.extract().unwrap();
                    let roundtripped_ninf: OrderedFloat<$float_type> = ninf_py.extract().unwrap();

                    assert_eq!(pinf, roundtripped_pinf);
                    assert_eq!(ninf, roundtripped_ninf);
                })
            }

            #[test]
            fn $zero_test() {
                let inner_pzero: $float_type = 0.0;
                let pzero = OrderedFloat(inner_pzero);

                let inner_nzero: $float_type = -0.0;
                let nzero = OrderedFloat(inner_nzero);

                Python::with_gil(|py| {
                    let pzero_py = pzero.into_pyobject(py).unwrap();
                    let nzero_py = nzero.into_pyobject(py).unwrap();

                    let locals = PyDict::new(py);
                    locals.set_item("pzero_py", &pzero_py).unwrap();
                    locals.set_item("nzero_py", &nzero_py).unwrap();

                    py.run(
                        &CString::new(
                            "\
                            assert pzero_py == 0.0\n\
                            assert nzero_py == -0.0",
                        )
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let roundtripped_pzero: OrderedFloat<$float_type> = pzero_py.extract().unwrap();
                    let roundtripped_nzero: OrderedFloat<$float_type> = nzero_py.extract().unwrap();

                    assert_eq!(pzero, roundtripped_pzero);
                    assert_eq!(nzero, roundtripped_nzero);
                })
            }

            #[test]
            fn $nan_test() {
                let inner_nan: $float_type = <$float_type>::NAN;
                let nan = OrderedFloat(inner_nan);

                Python::with_gil(|py| {
                    let nan_py = nan.into_pyobject(py).unwrap();

                    let locals = PyDict::new(py);
                    locals.set_item("nan_py", &nan_py).unwrap();

                    py.run(
                        &CString::new(
                            "\
                            import math\n\
                            assert math.isnan(nan_py)",
                        )
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let roundtripped_nan: OrderedFloat<$float_type> = nan_py.extract().unwrap();

                    assert_eq!(nan, roundtripped_nan);
                })
            }
        };
    }
    ordered_float_roundtrip_tests!(
        ordered_float_f32_standard,
        ordered_float_f32_wasm,
        ordered_float_f32_infinity,
        ordered_float_f32_zero,
        ordered_float_f32_nan,
        f32
    );
    ordered_float_roundtrip_tests!(
        ordered_float_f64_standard,
        ordered_float_f64_wasm,
        ordered_float_f64_infinity,
        ordered_float_f64_zero,
        ordered_float_f64_nan,
        f64
    );

    macro_rules! not_nan_roundtrip_tests {
        ($standard_test:ident, $wasm_test:ident, $infinity_test:ident, $zero_test:ident, $float_type:ty) => {
            #[cfg(not(target_arch = "wasm32"))]
            proptest! {
            #[test]
            fn $standard_test(inner_f: $float_type) {
                let f = NotNan::new(inner_f).unwrap();

                Python::with_gil(|py| {
                    let f_py = f.into_pyobject(py).unwrap();

                    let locals = PyDict::new(py);
                    locals.set_item("f_py", &f_py).unwrap();

                    py.run(
                        &CString::new(format!(
                            "import math\nassert math.isclose(f_py, {})",
                             inner_f as f64 // Always interpret the literal rs float value as f64
                                            // so that it's comparable with the python float
                        ))
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let roundtripped_f: NotNan<$float_type> = f_py.extract().unwrap();

                    assert_eq!(f, roundtripped_f);
                })
            }
            }

            #[cfg(target_arch = "wasm32")]
            fn $wasm_test() {
                let inner_f = 10.0;
                let f = NotNan::new(inner_f).unwrap();

                Python::with_gil(|py| {
                    let f_py = f.into_pyobject(py).unwrap();

                    let locals = PyDict::new(py);
                    locals.set_item("f_py", &f_py).unwrap();

                    py.run(
                        &CString::new(format!(
                            "import math\nassert math.isclose(f_py, {})",
                            inner_f as f64 // Always interpret the literal rs float value as f64
                                           // so that it's comparable with the python float
                        ))
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let roundtripped_f: NotNan<$float_type> = f_py.extract().unwrap();

                    assert_eq!(f, roundtripped_f);
                })
            }

            #[test]
            fn $infinity_test() {
                let inner_pinf = <$float_type>::INFINITY;
                let pinf = NotNan::new(inner_pinf).unwrap();

                let inner_ninf = <$float_type>::NEG_INFINITY;
                let ninf = NotNan::new(inner_ninf).unwrap();

                Python::with_gil(|py| {
                    let pinf_py = pinf.into_pyobject(py).unwrap();
                    let ninf_py = ninf.into_pyobject(py).unwrap();

                    let locals = PyDict::new(py);
                    locals.set_item("pinf_py", &pinf_py).unwrap();
                    locals.set_item("ninf_py", &ninf_py).unwrap();

                    py.run(
                        &CString::new(
                            "\
                            assert pinf_py == float('inf')\n\
                            assert ninf_py == float('-inf')",
                        )
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let roundtripped_pinf: NotNan<$float_type> = pinf_py.extract().unwrap();
                    let roundtripped_ninf: NotNan<$float_type> = ninf_py.extract().unwrap();

                    assert_eq!(pinf, roundtripped_pinf);
                    assert_eq!(ninf, roundtripped_ninf);
                })
            }

            #[test]
            fn $zero_test() {
                let inner_pzero: $float_type = 0.0;
                let pzero = NotNan::new(inner_pzero).unwrap();

                let inner_nzero: $float_type = -0.0;
                let nzero = NotNan::new(inner_nzero).unwrap();

                Python::with_gil(|py| {
                    let pzero_py = pzero.into_pyobject(py).unwrap();
                    let nzero_py = nzero.into_pyobject(py).unwrap();

                    let locals = PyDict::new(py);
                    locals.set_item("pzero_py", &pzero_py).unwrap();
                    locals.set_item("nzero_py", &nzero_py).unwrap();

                    py.run(
                        &CString::new(
                            "\
                            assert pzero_py == 0.0\n\
                            assert nzero_py == -0.0",
                        )
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let roundtripped_pzero: NotNan<$float_type> = pzero_py.extract().unwrap();
                    let roundtripped_nzero: NotNan<$float_type> = nzero_py.extract().unwrap();

                    assert_eq!(pzero, roundtripped_pzero);
                    assert_eq!(nzero, roundtripped_nzero);
                })
            }
        };
    }
    not_nan_roundtrip_tests!(
        not_nan_f32_standard,
        not_nan_f32_wasm,
        not_nan_f32_infinity,
        not_nan_f32_zero,
        f32
    );
    not_nan_roundtrip_tests!(
        not_nan_f64_standard,
        not_nan_f64_wasm,
        not_nan_f64_infinity,
        not_nan_f64_zero,
        f64
    );

    macro_rules! not_nan_pynan_tests {
        ($test_name:ident, $float_type:ty) => {
            #[test]
            fn $test_name() {
                Python::with_gil(|py| {
                    let locals = PyDict::new(py);

                    py.run(
                        &CString::new("nan_py = float('nan')").unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();

                    let nan_rs: PyResult<NotNan<$float_type>> =
                        locals.get_item("nan_py").unwrap().unwrap().extract();

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
                Python::with_gil(|py| {
                    let locals = PyDict::new(py);
                    py.run(
                        &CString::new(
                            "import sys\n\
                            max_float = sys.float_info.max",
                        )
                        .unwrap(),
                        None,
                        Some(&locals),
                    )
                    .unwrap();
                    let py_64 = locals.get_item("max_float").unwrap().unwrap();
                    let rs_64 = py_64.extract::<$wrapper<f32>>().unwrap();
                    // The python f64 is not representable in a rust f32
                    assert!(rs_64.is_infinite());
                })
            }
        };
    }
    py64_rs32!(ordered_float_f32, OrderedFloat, f32);
    py64_rs32!(ordered_float_f64, OrderedFloat, f64);
    py64_rs32!(not_nan_f32, NotNan, f32);
    py64_rs32!(not_nan_f64, NotNan, f64);
}
