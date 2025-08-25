#![cfg(feature = "num-complex")]

//!  Conversions to and from [num-complex](https://docs.rs/num-complex)’
//! [`Complex`]`<`[`f32`]`>` and [`Complex`]`<`[`f64`]`>`.
//!
//! num-complex’ [`Complex`] supports more operations than PyO3's [`PyComplex`]
//! and can be used with the rest of the Rust ecosystem.
//!
//! # Setup
//!
//! To use this feature, add this to your **`Cargo.toml`**:
//!
//! ```toml
//! [dependencies]
//! # change * to the latest versions
//! num-complex = "*"
#![doc = concat!("pyo3 = { version = \"", env!("CARGO_PKG_VERSION"),  "\", features = [\"num-complex\"] }")]
//! ```
//!
//! Note that you must use compatible versions of num-complex and PyO3.
//! The required num-complex version may vary based on the version of PyO3.
//!
//! # Examples
//!
//! Using [num-complex](https://docs.rs/num-complex) and [nalgebra](https://docs.rs/nalgebra)
//! to create a pyfunction that calculates the eigenvalues of a 2x2 matrix.
//! ```ignore
//! # // not tested because nalgebra isn't supported on msrv
//! # // please file an issue if it breaks!
//! use nalgebra::base::{dimension::Const, Matrix};
//! use num_complex::Complex;
//! use pyo3::prelude::*;
//!
//! type T = Complex<f64>;
//!
//! #[pyfunction]
//! fn get_eigenvalues(m11: T, m12: T, m21: T, m22: T) -> Vec<T> {
//!     let mat = Matrix::<T, Const<2>, Const<2>, _>::new(m11, m12, m21, m22);
//!
//!     match mat.eigenvalues() {
//!         Some(e) => e.data.as_slice().to_vec(),
//!         None => vec![],
//!     }
//! }
//!
//! #[pymodule]
//! fn my_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
//!     m.add_function(wrap_pyfunction!(get_eigenvalues, m)?)?;
//!     Ok(())
//! }
//! # // test
//! # use assert_approx_eq::assert_approx_eq;
//! # use nalgebra::ComplexField;
//! # use pyo3::types::PyComplex;
//! #
//! # fn main() -> PyResult<()> {
//! #     Python::attach(|py| -> PyResult<()> {
//! #         let module = PyModule::new(py, "my_module")?;
//! #
//! #         module.add_function(&wrap_pyfunction!(get_eigenvalues, module)?)?;
//! #
//! #         let m11 = PyComplex::from_doubles(py, 0_f64, -1_f64);
//! #         let m12 = PyComplex::from_doubles(py, 1_f64, 0_f64);
//! #         let m21 = PyComplex::from_doubles(py, 2_f64, -1_f64);
//! #         let m22 = PyComplex::from_doubles(py, -1_f64, 0_f64);
//! #
//! #         let result = module
//! #             .getattr("get_eigenvalues")?
//! #             .call1((m11, m12, m21, m22))?;
//! #         println!("eigenvalues: {:?}", result);
//! #
//! #         let result = result.extract::<Vec<T>>()?;
//! #         let e0 = result[0];
//! #         let e1 = result[1];
//! #
//! #         assert_approx_eq!(e0, Complex::new(1_f64, -1_f64));
//! #         assert_approx_eq!(e1, Complex::new(-2_f64, 0_f64));
//! #
//! #         Ok(())
//! #     })
//! # }
//! ```
//!
//! Python code:
//! ```python
//! from my_module import get_eigenvalues
//!
//! m11 = complex(0,-1)
//! m12 = complex(1,0)
//! m21 = complex(2,-1)
//! m22 = complex(-1,0)
//!
//! result = get_eigenvalues(m11,m12,m21,m22)
//! assert result == [complex(1,-1), complex(-2,0)]
//! ```
use crate::{
    ffi, ffi_ptr_ext::FfiPtrExt, types::PyComplex, Bound, FromPyObject, PyAny, PyErr, PyResult,
    Python,
};
use num_complex::Complex;
use std::ffi::c_double;

impl PyComplex {
    /// Creates a new Python `PyComplex` object from `num_complex`'s [`Complex`].
    pub fn from_complex_bound<F: Into<c_double>>(
        py: Python<'_>,
        complex: Complex<F>,
    ) -> Bound<'_, PyComplex> {
        unsafe {
            ffi::PyComplex_FromDoubles(complex.re.into(), complex.im.into())
                .assume_owned(py)
                .cast_into_unchecked()
        }
    }
}

macro_rules! complex_conversion {
    ($float: ty) => {
        #[cfg_attr(docsrs, doc(cfg(feature = "num-complex")))]
        impl<'py> crate::conversion::IntoPyObject<'py> for Complex<$float> {
            type Target = PyComplex;
            type Output = Bound<'py, Self::Target>;
            type Error = std::convert::Infallible;

            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                unsafe {
                    Ok(
                        ffi::PyComplex_FromDoubles(self.re as c_double, self.im as c_double)
                            .assume_owned(py)
                            .cast_into_unchecked(),
                    )
                }
            }
        }

        #[cfg_attr(docsrs, doc(cfg(feature = "num-complex")))]
        impl<'py> crate::conversion::IntoPyObject<'py> for &Complex<$float> {
            type Target = PyComplex;
            type Output = Bound<'py, Self::Target>;
            type Error = std::convert::Infallible;

            #[inline]
            fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
                (*self).into_pyobject(py)
            }
        }

        #[cfg_attr(docsrs, doc(cfg(feature = "num-complex")))]
        impl FromPyObject<'_> for Complex<$float> {
            fn extract_bound(obj: &Bound<'_, PyAny>) -> PyResult<Complex<$float>> {
                #[cfg(not(any(Py_LIMITED_API, PyPy)))]
                unsafe {
                    let val = ffi::PyComplex_AsCComplex(obj.as_ptr());
                    if val.real == -1.0 {
                        if let Some(err) = PyErr::take(obj.py()) {
                            return Err(err);
                        }
                    }
                    Ok(Complex::new(val.real as $float, val.imag as $float))
                }

                #[cfg(any(Py_LIMITED_API, PyPy))]
                unsafe {
                    use $crate::types::any::PyAnyMethods;
                    let complex;
                    let obj = if obj.is_instance_of::<PyComplex>() {
                        obj
                    } else if let Some(method) =
                        obj.lookup_special(crate::intern!(obj.py(), "__complex__"))?
                    {
                        complex = method.call0()?;
                        &complex
                    } else {
                        // `obj` might still implement `__float__` or `__index__`, which will be
                        // handled by `PyComplex_{Real,Imag}AsDouble`, including propagating any
                        // errors if those methods don't exist / raise exceptions.
                        obj
                    };
                    let ptr = obj.as_ptr();
                    let real = ffi::PyComplex_RealAsDouble(ptr);
                    if real == -1.0 {
                        if let Some(err) = PyErr::take(obj.py()) {
                            return Err(err);
                        }
                    }
                    let imag = ffi::PyComplex_ImagAsDouble(ptr);
                    Ok(Complex::new(real as $float, imag as $float))
                }
            }
        }
    };
}
complex_conversion!(f32);
complex_conversion!(f64);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::generate_unique_module_name;
    use crate::types::PyAnyMethods as _;
    use crate::types::{complex::PyComplexMethods, PyModule};
    use crate::IntoPyObject;
    use pyo3_ffi::c_str;

    #[test]
    fn from_complex() {
        Python::attach(|py| {
            let complex = Complex::new(3.0, 1.2);
            let py_c = PyComplex::from_complex_bound(py, complex);
            assert_eq!(py_c.real(), 3.0);
            assert_eq!(py_c.imag(), 1.2);
        });
    }
    #[test]
    fn to_from_complex() {
        Python::attach(|py| {
            let val = Complex::new(3.0f64, 1.2);
            let obj = val.into_pyobject(py).unwrap();
            assert_eq!(obj.extract::<Complex<f64>>().unwrap(), val);
        });
    }
    #[test]
    fn from_complex_err() {
        Python::attach(|py| {
            let obj = vec![1i32].into_pyobject(py).unwrap();
            assert!(obj.extract::<Complex<f64>>().is_err());
        });
    }
    #[test]
    fn from_python_magic() {
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                c_str!(
                    r#"
class A:
    def __complex__(self): return 3.0+1.2j
class B:
    def __float__(self): return 3.0
class C:
    def __index__(self): return 3
                "#
                ),
                c_str!("test.py"),
                &generate_unique_module_name("test"),
            )
            .unwrap();
            let from_complex = module.getattr("A").unwrap().call0().unwrap();
            assert_eq!(
                from_complex.extract::<Complex<f64>>().unwrap(),
                Complex::new(3.0, 1.2)
            );
            let from_float = module.getattr("B").unwrap().call0().unwrap();
            assert_eq!(
                from_float.extract::<Complex<f64>>().unwrap(),
                Complex::new(3.0, 0.0)
            );
            // Before Python 3.8, `__index__` wasn't tried by `float`/`complex`.
            #[cfg(Py_3_8)]
            {
                let from_index = module.getattr("C").unwrap().call0().unwrap();
                assert_eq!(
                    from_index.extract::<Complex<f64>>().unwrap(),
                    Complex::new(3.0, 0.0)
                );
            }
        })
    }
    #[test]
    fn from_python_inherited_magic() {
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                c_str!(
                    r#"
class First: pass
class ComplexMixin:
    def __complex__(self): return 3.0+1.2j
class FloatMixin:
    def __float__(self): return 3.0
class IndexMixin:
    def __index__(self): return 3
class A(First, ComplexMixin): pass
class B(First, FloatMixin): pass
class C(First, IndexMixin): pass
                "#
                ),
                c_str!("test.py"),
                &generate_unique_module_name("test"),
            )
            .unwrap();
            let from_complex = module.getattr("A").unwrap().call0().unwrap();
            assert_eq!(
                from_complex.extract::<Complex<f64>>().unwrap(),
                Complex::new(3.0, 1.2)
            );
            let from_float = module.getattr("B").unwrap().call0().unwrap();
            assert_eq!(
                from_float.extract::<Complex<f64>>().unwrap(),
                Complex::new(3.0, 0.0)
            );
            #[cfg(Py_3_8)]
            {
                let from_index = module.getattr("C").unwrap().call0().unwrap();
                assert_eq!(
                    from_index.extract::<Complex<f64>>().unwrap(),
                    Complex::new(3.0, 0.0)
                );
            }
        })
    }
    #[test]
    fn from_python_noncallable_descriptor_magic() {
        // Functions and lambdas implement the descriptor protocol in a way that makes
        // `type(inst).attr(inst)` equivalent to `inst.attr()` for methods, but this isn't the only
        // way the descriptor protocol might be implemented.
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                c_str!(
                    r#"
class A:
    @property
    def __complex__(self):
        return lambda: 3.0+1.2j
                "#
                ),
                c_str!("test.py"),
                &generate_unique_module_name("test"),
            )
            .unwrap();
            let obj = module.getattr("A").unwrap().call0().unwrap();
            assert_eq!(
                obj.extract::<Complex<f64>>().unwrap(),
                Complex::new(3.0, 1.2)
            );
        })
    }
    #[test]
    fn from_python_nondescriptor_magic() {
        // Magic methods don't need to implement the descriptor protocol, if they're callable.
        Python::attach(|py| {
            let module = PyModule::from_code(
                py,
                c_str!(
                    r#"
class MyComplex:
    def __call__(self): return 3.0+1.2j
class A:
    __complex__ = MyComplex()
                "#
                ),
                c_str!("test.py"),
                &generate_unique_module_name("test"),
            )
            .unwrap();
            let obj = module.getattr("A").unwrap().call0().unwrap();
            assert_eq!(
                obj.extract::<Complex<f64>>().unwrap(),
                Complex::new(3.0, 1.2)
            );
        })
    }
}
