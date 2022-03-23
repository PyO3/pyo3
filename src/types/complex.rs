use crate::{ffi, AsPyPointer, PyAny, Python};
use std::os::raw::c_double;

/// Represents a Python [`complex`](https://docs.python.org/3/library/functions.html#complex) object.
///
/// Note that `PyComplex` supports only basic operations. For advanced operations
/// consider using [num-complex](https://docs.rs/num-complex)'s [`Complex`] type instead.
/// This optional dependency can be activated with the `num-complex` feature flag.
///
/// [`Complex`]: https://docs.rs/num-complex/latest/num_complex/struct.Complex.html
#[repr(transparent)]
pub struct PyComplex(PyAny);

pyobject_native_type!(
    PyComplex,
    ffi::PyComplexObject,
    ffi::PyComplex_Type,
    #checkfunction=ffi::PyComplex_Check
);

impl PyComplex {
    /// Creates a new `PyComplex` from the given real and imaginary values.
    pub fn from_doubles(py: Python<'_>, real: c_double, imag: c_double) -> &PyComplex {
        unsafe {
            let ptr = ffi::PyComplex_FromDoubles(real, imag);
            py.from_owned_ptr(ptr)
        }
    }
    /// Returns the real part of the complex number.
    pub fn real(&self) -> c_double {
        unsafe { ffi::PyComplex_RealAsDouble(self.as_ptr()) }
    }
    /// Returns the imaginary part of the complex number.
    pub fn imag(&self) -> c_double {
        unsafe { ffi::PyComplex_ImagAsDouble(self.as_ptr()) }
    }
}

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
mod not_limited_impls {
    use super::*;
    use std::ops::{Add, Div, Mul, Neg, Sub};

    impl PyComplex {
        /// Returns `|self|`.
        pub fn abs(&self) -> c_double {
            unsafe {
                let val = (*(self.as_ptr() as *mut ffi::PyComplexObject)).cval;
                ffi::_Py_c_abs(val)
            }
        }
        /// Returns `self` raised to the power of `other`.
        pub fn pow(&self, other: &PyComplex) -> &PyComplex {
            unsafe {
                self.py()
                    .from_owned_ptr(complex_operation(self, other, ffi::_Py_c_pow))
            }
        }
    }

    #[inline(always)]
    unsafe fn complex_operation(
        l: &PyComplex,
        r: &PyComplex,
        operation: unsafe extern "C" fn(ffi::Py_complex, ffi::Py_complex) -> ffi::Py_complex,
    ) -> *mut ffi::PyObject {
        let l_val = (*(l.as_ptr() as *mut ffi::PyComplexObject)).cval;
        let r_val = (*(r.as_ptr() as *mut ffi::PyComplexObject)).cval;
        ffi::PyComplex_FromCComplex(operation(l_val, r_val))
    }

    impl<'py> Add for &'py PyComplex {
        type Output = &'py PyComplex;
        fn add(self, other: &'py PyComplex) -> &'py PyComplex {
            unsafe {
                self.py()
                    .from_owned_ptr(complex_operation(self, other, ffi::_Py_c_sum))
            }
        }
    }

    impl<'py> Sub for &'py PyComplex {
        type Output = &'py PyComplex;
        fn sub(self, other: &'py PyComplex) -> &'py PyComplex {
            unsafe {
                self.py()
                    .from_owned_ptr(complex_operation(self, other, ffi::_Py_c_diff))
            }
        }
    }

    impl<'py> Mul for &'py PyComplex {
        type Output = &'py PyComplex;
        fn mul(self, other: &'py PyComplex) -> &'py PyComplex {
            unsafe {
                self.py()
                    .from_owned_ptr(complex_operation(self, other, ffi::_Py_c_prod))
            }
        }
    }

    impl<'py> Div for &'py PyComplex {
        type Output = &'py PyComplex;
        fn div(self, other: &'py PyComplex) -> &'py PyComplex {
            unsafe {
                self.py()
                    .from_owned_ptr(complex_operation(self, other, ffi::_Py_c_quot))
            }
        }
    }

    impl<'py> Neg for &'py PyComplex {
        type Output = &'py PyComplex;
        fn neg(self) -> &'py PyComplex {
            unsafe {
                let val = (*(self.as_ptr() as *mut ffi::PyComplexObject)).cval;
                self.py()
                    .from_owned_ptr(ffi::PyComplex_FromCComplex(ffi::_Py_c_neg(val)))
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::PyComplex;
        use crate::Python;
        use assert_approx_eq::assert_approx_eq;

        #[test]
        fn test_add() {
            Python::with_gil(|py| {
                let l = PyComplex::from_doubles(py, 3.0, 1.2);
                let r = PyComplex::from_doubles(py, 1.0, 2.6);
                let res = l + r;
                assert_approx_eq!(res.real(), 4.0);
                assert_approx_eq!(res.imag(), 3.8);
            });
        }

        #[test]
        fn test_sub() {
            Python::with_gil(|py| {
                let l = PyComplex::from_doubles(py, 3.0, 1.2);
                let r = PyComplex::from_doubles(py, 1.0, 2.6);
                let res = l - r;
                assert_approx_eq!(res.real(), 2.0);
                assert_approx_eq!(res.imag(), -1.4);
            });
        }

        #[test]
        fn test_mul() {
            Python::with_gil(|py| {
                let l = PyComplex::from_doubles(py, 3.0, 1.2);
                let r = PyComplex::from_doubles(py, 1.0, 2.6);
                let res = l * r;
                assert_approx_eq!(res.real(), -0.12);
                assert_approx_eq!(res.imag(), 9.0);
            });
        }

        #[test]
        fn test_div() {
            Python::with_gil(|py| {
                let l = PyComplex::from_doubles(py, 3.0, 1.2);
                let r = PyComplex::from_doubles(py, 1.0, 2.6);
                let res = l / r;
                assert_approx_eq!(res.real(), 0.788_659_793_814_432_9);
                assert_approx_eq!(res.imag(), -0.850_515_463_917_525_7);
            });
        }

        #[test]
        fn test_neg() {
            Python::with_gil(|py| {
                let val = PyComplex::from_doubles(py, 3.0, 1.2);
                let res = -val;
                assert_approx_eq!(res.real(), -3.0);
                assert_approx_eq!(res.imag(), -1.2);
            });
        }

        #[test]
        fn test_abs() {
            Python::with_gil(|py| {
                let val = PyComplex::from_doubles(py, 3.0, 1.2);
                assert_approx_eq!(val.abs(), 3.231_098_884_280_702_2);
            });
        }

        #[test]
        fn test_pow() {
            Python::with_gil(|py| {
                let l = PyComplex::from_doubles(py, 3.0, 1.2);
                let r = PyComplex::from_doubles(py, 1.2, 2.6);
                let val = l.pow(r);
                assert_approx_eq!(val.real(), -1.419_309_997_016_603_7);
                assert_approx_eq!(val.imag(), -0.541_297_466_033_544_6);
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PyComplex;
    use crate::Python;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_from_double() {
        use assert_approx_eq::assert_approx_eq;

        Python::with_gil(|py| {
            let complex = PyComplex::from_doubles(py, 3.0, 1.2);
            assert_approx_eq!(complex.real(), 3.0);
            assert_approx_eq!(complex.imag(), 1.2);
        });
    }
}
