#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
use crate::py_result_ext::PyResultExt;
#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
use crate::types::any::PyAnyMethods;
use crate::{ffi, Bound, PyAny, Python};
use std::ffi::c_double;

/// Represents a Python [`complex`](https://docs.python.org/3/library/functions.html#complex) object.
///
/// Values of this type are accessed via PyO3's smart pointers, e.g. as
/// [`Py<PyComplex>`][crate::Py] or [`Bound<'py, PyComplex>`][Bound].
///
/// For APIs available on `complex` objects, see the [`PyComplexMethods`] trait which is implemented for
/// [`Bound<'py, PyComplex>`][Bound].
///
/// Note that `PyComplex` supports only basic operations. For advanced operations
/// consider using [num-complex](https://docs.rs/num-complex)'s [`Complex`] type instead.
/// This optional dependency can be activated with the `num-complex` feature flag.
///
/// [`Complex`]: https://docs.rs/num-complex/latest/num_complex/struct.Complex.html
#[repr(transparent)]
pub struct PyComplex(PyAny);

pyobject_subclassable_native_type!(PyComplex, ffi::PyComplexObject);

pyobject_native_type!(
    PyComplex,
    ffi::PyComplexObject,
    pyobject_native_static_type_object!(ffi::PyComplex_Type),
    #checkfunction=ffi::PyComplex_Check
);

impl PyComplex {
    /// Creates a new `PyComplex` from the given real and imaginary values.
    pub fn from_doubles(py: Python<'_>, real: c_double, imag: c_double) -> Bound<'_, PyComplex> {
        use crate::ffi_ptr_ext::FfiPtrExt;
        unsafe {
            ffi::PyComplex_FromDoubles(real, imag)
                .assume_owned(py)
                .cast_into_unchecked()
        }
    }
}

#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
mod not_limited_impls {
    use crate::Borrowed;

    use super::*;
    use std::ops::{Add, Div, Mul, Neg, Sub};

    macro_rules! bin_ops {
        ($trait:ident, $fn:ident, $op:tt) => {
            impl<'py> $trait for Borrowed<'_, 'py, PyComplex> {
                type Output = Bound<'py, PyComplex>;
                fn $fn(self, other: Self) -> Self::Output {
                    PyAnyMethods::$fn(self.as_any(), other)
                    .cast_into().expect(
                        concat!("Complex method ",
                            stringify!($fn),
                            " failed.")
                        )
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

    bin_ops!(Add, add, +);
    bin_ops!(Sub, sub, -);
    bin_ops!(Mul, mul, *);
    bin_ops!(Div, div, /);

    impl<'py> Neg for Borrowed<'_, 'py, PyComplex> {
        type Output = Bound<'py, PyComplex>;
        fn neg(self) -> Self::Output {
            PyAnyMethods::neg(self.as_any())
                .cast_into()
                .expect("Complex method __neg__ failed.")
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
            Python::attach(|py| {
                let l = PyComplex::from_doubles(py, 3.0, 1.2);
                let r = PyComplex::from_doubles(py, 1.0, 2.6);
                let res = l + r;
                assert_approx_eq!(res.real(), 4.0);
                assert_approx_eq!(res.imag(), 3.8);
            });
        }

        #[test]
        fn test_sub() {
            Python::attach(|py| {
                let l = PyComplex::from_doubles(py, 3.0, 1.2);
                let r = PyComplex::from_doubles(py, 1.0, 2.6);
                let res = l - r;
                assert_approx_eq!(res.real(), 2.0);
                assert_approx_eq!(res.imag(), -1.4);
            });
        }

        #[test]
        fn test_mul() {
            Python::attach(|py| {
                let l = PyComplex::from_doubles(py, 3.0, 1.2);
                let r = PyComplex::from_doubles(py, 1.0, 2.6);
                let res = l * r;
                assert_approx_eq!(res.real(), -0.12);
                assert_approx_eq!(res.imag(), 9.0);
            });
        }

        #[test]
        fn test_div() {
            Python::attach(|py| {
                let l = PyComplex::from_doubles(py, 3.0, 1.2);
                let r = PyComplex::from_doubles(py, 1.0, 2.6);
                let res = l / r;
                assert_approx_eq!(res.real(), 0.788_659_793_814_432_9);
                assert_approx_eq!(res.imag(), -0.850_515_463_917_525_7);
            });
        }

        #[test]
        fn test_neg() {
            Python::attach(|py| {
                let val = PyComplex::from_doubles(py, 3.0, 1.2);
                let res = -val;
                assert_approx_eq!(res.real(), -3.0);
                assert_approx_eq!(res.imag(), -1.2);
            });
        }

        #[test]
        fn test_abs() {
            Python::attach(|py| {
                let val = PyComplex::from_doubles(py, 3.0, 1.2);
                assert_approx_eq!(val.abs(), 3.231_098_884_280_702_2);
            });
        }

        #[test]
        fn test_pow() {
            Python::attach(|py| {
                let l = PyComplex::from_doubles(py, 3.0, 1.2);
                let r = PyComplex::from_doubles(py, 1.2, 2.6);
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
    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    fn abs(&self) -> c_double;
    /// Returns `self` raised to the power of `other`.
    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    fn pow(&self, other: &Bound<'py, PyComplex>) -> Bound<'py, PyComplex>;
}

impl<'py> PyComplexMethods<'py> for Bound<'py, PyComplex> {
    fn real(&self) -> c_double {
        unsafe { ffi::PyComplex_RealAsDouble(self.as_ptr()) }
    }

    fn imag(&self) -> c_double {
        unsafe { ffi::PyComplex_ImagAsDouble(self.as_ptr()) }
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    fn abs(&self) -> c_double {
        PyAnyMethods::abs(self.as_any())
            .cast_into()
            .expect("Complex method __abs__ failed.")
            .extract()
            .expect("Failed to extract to c double.")
    }

    #[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
    fn pow(&self, other: &Bound<'py, PyComplex>) -> Bound<'py, PyComplex> {
        Python::attach(|py| {
            PyAnyMethods::pow(self.as_any(), other, py.None())
                .cast_into()
                .expect("Complex method __pow__ failed.")
        })
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

        Python::attach(|py| {
            let complex = PyComplex::from_doubles(py, 3.0, 1.2);
            assert_approx_eq!(complex.real(), 3.0);
            assert_approx_eq!(complex.imag(), 1.2);
        });
    }
}
