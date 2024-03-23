use crate::{ffi, Bound, PyAny, PyNativeType, Python};
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
    pyobject_native_static_type_object!(ffi::PyComplex_Type),
    #checkfunction=ffi::PyComplex_Check
);

impl PyComplex {
    /// Deprecated form of [`PyComplex::from_doubles_bound`]
    #[cfg_attr(
        not(feature = "gil-refs"),
        deprecated(
            since = "0.21.0",
            note = "`PyComplex::from_doubles` will be replaced by `PyComplex::from_doubles_bound` in a future PyO3 version"
        )
    )]
    pub fn from_doubles(py: Python<'_>, real: c_double, imag: c_double) -> &PyComplex {
        Self::from_doubles_bound(py, real, imag).into_gil_ref()
    }
    /// Creates a new `PyComplex` from the given real and imaginary values.
    pub fn from_doubles_bound(
        py: Python<'_>,
        real: c_double,
        imag: c_double,
    ) -> Bound<'_, PyComplex> {
        use crate::ffi_ptr_ext::FfiPtrExt;
        unsafe {
            ffi::PyComplex_FromDoubles(real, imag)
                .assume_owned(py)
                .downcast_into_unchecked()
        }
    }
    /// Returns the real part of the complex number.
    pub fn real(&self) -> c_double {
        self.as_borrowed().real()
    }
    /// Returns the imaginary part of the complex number.
    pub fn imag(&self) -> c_double {
        self.as_borrowed().imag()
    }
}

#[cfg(not(any(Py_LIMITED_API, PyPy)))]
mod not_limited_impls {
    use crate::ffi_ptr_ext::FfiPtrExt;
    use crate::Borrowed;

    use super::*;
    use std::ops::{Add, Div, Mul, Neg, Sub};

    impl PyComplex {
        /// Returns `|self|`.
        pub fn abs(&self) -> c_double {
            self.as_borrowed().abs()
        }
        /// Returns `self` raised to the power of `other`.
        pub fn pow<'py>(&'py self, other: &'py PyComplex) -> &'py PyComplex {
            self.as_borrowed().pow(&other.as_borrowed()).into_gil_ref()
        }
    }

    #[inline(always)]
    pub(super) unsafe fn complex_operation<'py>(
        l: Borrowed<'_, 'py, PyComplex>,
        r: Borrowed<'_, 'py, PyComplex>,
        operation: unsafe extern "C" fn(ffi::Py_complex, ffi::Py_complex) -> ffi::Py_complex,
    ) -> *mut ffi::PyObject {
        let l_val = (*l.as_ptr().cast::<ffi::PyComplexObject>()).cval;
        let r_val = (*r.as_ptr().cast::<ffi::PyComplexObject>()).cval;
        ffi::PyComplex_FromCComplex(operation(l_val, r_val))
    }

    macro_rules! bin_ops {
        ($trait:ident, $fn:ident, $op:tt, $ffi:path) => {
            impl<'py> $trait for Borrowed<'_, 'py, PyComplex> {
                type Output = Bound<'py, PyComplex>;
                fn $fn(self, other: Self) -> Self::Output {
                    unsafe {
                        complex_operation(self, other, $ffi)
                            .assume_owned(self.py())
                            .downcast_into_unchecked()
                    }
                }
            }

            impl<'py> $trait for &'py PyComplex {
                type Output = &'py PyComplex;
                fn $fn(self, other: &'py PyComplex) -> &'py PyComplex {
                    (self.as_borrowed() $op other.as_borrowed()).into_gil_ref()
                }
            }

            impl<'py> $trait for &Bound<'py, PyComplex> {
                type Output = Bound<'py, PyComplex>;
                fn $fn(self, other: &Bound<'py, PyComplex>) -> Bound<'py, PyComplex> {
                    self.as_borrowed() $op other.as_borrowed()
                }
            }

            impl<'py> $trait<Bound<'py, PyComplex>> for &Bound<'py, PyComplex> {
                type Output = Bound<'py, PyComplex>;
                fn $fn(self, other: Bound<'py, PyComplex>) -> Bound<'py, PyComplex> {
                    self.as_borrowed() $op other.as_borrowed()
                }
            }

            impl<'py> $trait for Bound<'py, PyComplex> {
                type Output = Bound<'py, PyComplex>;
                fn $fn(self, other: Bound<'py, PyComplex>) -> Bound<'py, PyComplex> {
                    self.as_borrowed() $op other.as_borrowed()
                }
            }

            impl<'py> $trait<&Self> for Bound<'py, PyComplex> {
                type Output = Bound<'py, PyComplex>;
                fn $fn(self, other: &Bound<'py, PyComplex>) -> Bound<'py, PyComplex> {
                    self.as_borrowed() $op other.as_borrowed()
                }
            }
        };
    }

    bin_ops!(Add, add, +, ffi::_Py_c_sum);
    bin_ops!(Sub, sub, -, ffi::_Py_c_diff);
    bin_ops!(Mul, mul, *, ffi::_Py_c_prod);
    bin_ops!(Div, div, /, ffi::_Py_c_quot);

    impl<'py> Neg for &'py PyComplex {
        type Output = &'py PyComplex;
        fn neg(self) -> &'py PyComplex {
            (-self.as_borrowed()).into_gil_ref()
        }
    }

    impl<'py> Neg for Borrowed<'_, 'py, PyComplex> {
        type Output = Bound<'py, PyComplex>;
        fn neg(self) -> Self::Output {
            unsafe {
                let val = (*self.as_ptr().cast::<ffi::PyComplexObject>()).cval;
                ffi::PyComplex_FromCComplex(ffi::_Py_c_neg(val))
                    .assume_owned(self.py())
                    .downcast_into_unchecked()
            }
        }
    }

    impl<'py> Neg for &Bound<'py, PyComplex> {
        type Output = Bound<'py, PyComplex>;
        fn neg(self) -> Bound<'py, PyComplex> {
            -self.as_borrowed()
        }
    }

    impl<'py> Neg for Bound<'py, PyComplex> {
        type Output = Bound<'py, PyComplex>;
        fn neg(self) -> Bound<'py, PyComplex> {
            -self.as_borrowed()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::PyComplex;
        use crate::{types::complex::PyComplexMethods, Python};
        use assert_approx_eq::assert_approx_eq;

        #[test]
        fn test_add() {
            Python::with_gil(|py| {
                let l = PyComplex::from_doubles_bound(py, 3.0, 1.2);
                let r = PyComplex::from_doubles_bound(py, 1.0, 2.6);
                let res = l + r;
                assert_approx_eq!(res.real(), 4.0);
                assert_approx_eq!(res.imag(), 3.8);
            });
        }

        #[test]
        fn test_sub() {
            Python::with_gil(|py| {
                let l = PyComplex::from_doubles_bound(py, 3.0, 1.2);
                let r = PyComplex::from_doubles_bound(py, 1.0, 2.6);
                let res = l - r;
                assert_approx_eq!(res.real(), 2.0);
                assert_approx_eq!(res.imag(), -1.4);
            });
        }

        #[test]
        fn test_mul() {
            Python::with_gil(|py| {
                let l = PyComplex::from_doubles_bound(py, 3.0, 1.2);
                let r = PyComplex::from_doubles_bound(py, 1.0, 2.6);
                let res = l * r;
                assert_approx_eq!(res.real(), -0.12);
                assert_approx_eq!(res.imag(), 9.0);
            });
        }

        #[test]
        fn test_div() {
            Python::with_gil(|py| {
                let l = PyComplex::from_doubles_bound(py, 3.0, 1.2);
                let r = PyComplex::from_doubles_bound(py, 1.0, 2.6);
                let res = l / r;
                assert_approx_eq!(res.real(), 0.788_659_793_814_432_9);
                assert_approx_eq!(res.imag(), -0.850_515_463_917_525_7);
            });
        }

        #[test]
        fn test_neg() {
            Python::with_gil(|py| {
                let val = PyComplex::from_doubles_bound(py, 3.0, 1.2);
                let res = -val;
                assert_approx_eq!(res.real(), -3.0);
                assert_approx_eq!(res.imag(), -1.2);
            });
        }

        #[test]
        fn test_abs() {
            Python::with_gil(|py| {
                let val = PyComplex::from_doubles_bound(py, 3.0, 1.2);
                assert_approx_eq!(val.abs(), 3.231_098_884_280_702_2);
            });
        }

        #[test]
        fn test_pow() {
            Python::with_gil(|py| {
                let l = PyComplex::from_doubles_bound(py, 3.0, 1.2);
                let r = PyComplex::from_doubles_bound(py, 1.2, 2.6);
                let val = l.pow(&r);
                assert_approx_eq!(val.real(), -1.419_309_997_016_603_7);
                assert_approx_eq!(val.imag(), -0.541_297_466_033_544_6);
            });
        }
    }
}

/// Implementation of functionality for [`PyComplex`].
///
/// These methods are defined for the `Bound<'py, PyComplex>` smart pointer, so to use method call
/// syntax these methods are separated into a trait, because stable Rust does not yet support
/// `arbitrary_self_types`.
#[doc(alias = "PyComplex")]
pub trait PyComplexMethods<'py>: crate::sealed::Sealed {
    /// Returns the real part of the complex number.
    fn real(&self) -> c_double;
    /// Returns the imaginary part of the complex number.
    fn imag(&self) -> c_double;
    /// Returns `|self|`.
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    fn abs(&self) -> c_double;
    /// Returns `self` raised to the power of `other`.
    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    fn pow(&self, other: &Bound<'py, PyComplex>) -> Bound<'py, PyComplex>;
}

impl<'py> PyComplexMethods<'py> for Bound<'py, PyComplex> {
    fn real(&self) -> c_double {
        unsafe { ffi::PyComplex_RealAsDouble(self.as_ptr()) }
    }

    fn imag(&self) -> c_double {
        unsafe { ffi::PyComplex_ImagAsDouble(self.as_ptr()) }
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    fn abs(&self) -> c_double {
        unsafe {
            let val = (*self.as_ptr().cast::<ffi::PyComplexObject>()).cval;
            ffi::_Py_c_abs(val)
        }
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy)))]
    fn pow(&self, other: &Bound<'py, PyComplex>) -> Bound<'py, PyComplex> {
        use crate::ffi_ptr_ext::FfiPtrExt;
        unsafe {
            not_limited_impls::complex_operation(
                self.as_borrowed(),
                other.as_borrowed(),
                ffi::_Py_c_pow,
            )
            .assume_owned(self.py())
            .downcast_into_unchecked()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PyComplex;
    use crate::{types::complex::PyComplexMethods, Python};
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_from_double() {
        use assert_approx_eq::assert_approx_eq;

        Python::with_gil(|py| {
            let complex = PyComplex::from_doubles_bound(py, 3.0, 1.2);
            assert_approx_eq!(complex.real(), 3.0);
            assert_approx_eq!(complex.imag(), 1.2);
        });
    }
}
