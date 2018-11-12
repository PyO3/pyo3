use crate::ffi;
use crate::instance::PyObjectWithGIL;
use crate::object::PyObject;
use crate::python::{Python, ToPyPointer};
#[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
use std::ops::*;
use std::os::raw::c_double;

/// Represents a Python `complex`.
#[repr(transparent)]
pub struct PyComplex(PyObject);

pyobject_native_type!(PyComplex, ffi::PyComplex_Type, ffi::PyComplex_Check);

impl PyComplex {
    /// Creates a new Python `PyComplex` object, from its real and imaginary values.
    pub fn from_doubles<'p>(py: Python<'p>, real: c_double, imag: c_double) -> &'p PyComplex {
        unsafe {
            let ptr = ffi::PyComplex_FromDoubles(real, imag);
            py.from_owned_ptr(ptr)
        }
    }
    /// Returns the real value of `PyComplex`.
    pub fn real(&self) -> c_double {
        unsafe { ffi::PyComplex_RealAsDouble(self.as_ptr()) }
    }
    /// Returns the imaginary value of `PyComplex`.
    pub fn imag(&self) -> c_double {
        unsafe { ffi::PyComplex_ImagAsDouble(self.as_ptr()) }
    }
    /// Returns `|self|`.
    #[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
    pub fn abs(&self) -> c_double {
        unsafe {
            let val = (*(self.as_ptr() as *mut ffi::PyComplexObject)).cval;
            ffi::_Py_c_abs(val)
        }
    }
    /// Returns `self ** other`
    #[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
    pub fn pow(&self, other: &PyComplex) -> &PyComplex {
        unsafe {
            self.py()
                .from_owned_ptr(complex_operation(self, other, ffi::_Py_c_pow))
        }
    }
}

#[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
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

#[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
impl<'py> Add for &'py PyComplex {
    type Output = &'py PyComplex;
    fn add(self, other: &'py PyComplex) -> &'py PyComplex {
        unsafe {
            self.py()
                .from_owned_ptr(complex_operation(self, other, ffi::_Py_c_sum))
        }
    }
}

#[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
impl<'py> Sub for &'py PyComplex {
    type Output = &'py PyComplex;
    fn sub(self, other: &'py PyComplex) -> &'py PyComplex {
        unsafe {
            self.py()
                .from_owned_ptr(complex_operation(self, other, ffi::_Py_c_diff))
        }
    }
}

#[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
impl<'py> Mul for &'py PyComplex {
    type Output = &'py PyComplex;
    fn mul(self, other: &'py PyComplex) -> &'py PyComplex {
        unsafe {
            self.py()
                .from_owned_ptr(complex_operation(self, other, ffi::_Py_c_prod))
        }
    }
}

#[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
impl<'py> Div for &'py PyComplex {
    type Output = &'py PyComplex;
    fn div(self, other: &'py PyComplex) -> &'py PyComplex {
        unsafe {
            self.py()
                .from_owned_ptr(complex_operation(self, other, ffi::_Py_c_quot))
        }
    }
}

#[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
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

#[cfg(feature = "num-complex")]
mod complex_conversion {
    extern crate num_complex;
    use self::num_complex::Complex;
    use super::*;
    use conversion::{FromPyObject, IntoPyObject, ToPyObject};
    use err::PyErr;
    use types::PyObjectRef;
    use PyResult;
    impl PyComplex {
        /// Creates a new Python `PyComplex` object from num_complex::Complex.
        pub fn from_complex<'py, F: Into<c_double>>(
            py: Python<'py>,
            complex: Complex<F>,
        ) -> &'py PyComplex {
            unsafe {
                let ptr = ffi::PyComplex_FromDoubles(complex.re.into(), complex.im.into());
                py.from_owned_ptr(ptr)
            }
        }
    }
    macro_rules! complex_conversion {
        ($float: ty) => {
            impl ToPyObject for Complex<$float> {
                #[inline]
                fn to_object(&self, py: Python) -> PyObject {
                    IntoPyObject::into_object(self.to_owned(), py)
                }
            }
            impl IntoPyObject for Complex<$float> {
                fn into_object(self, py: Python) -> PyObject {
                    unsafe {
                        let raw_obj =
                            ffi::PyComplex_FromDoubles(self.re as c_double, self.im as c_double);
                        PyObject::from_owned_ptr_or_panic(py, raw_obj)
                    }
                }
            }
            #[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
            impl<'source> FromPyObject<'source> for Complex<$float> {
                fn extract(obj: &'source PyObjectRef) -> PyResult<Complex<$float>> {
                    unsafe {
                        let val = ffi::PyComplex_AsCComplex(obj.as_ptr());
                        if val.real == -1.0 && PyErr::occurred(obj.py()) {
                            Err(PyErr::fetch(obj.py()))
                        } else {
                            Ok(Complex::new(val.real as $float, val.imag as $float))
                        }
                    }
                }
            }
            #[cfg(all(Py_LIMITED_API, Py_3))]
            impl<'source> FromPyObject<'source> for Complex<$float> {
                fn extract(obj: &'source PyObjectRef) -> PyResult<Complex<$float>> {
                    unsafe {
                        let ptr = obj.as_ptr();
                        let real = ffi::PyComplex_RealAsDouble(ptr);
                        if real == -1.0 && PyErr::occurred(obj.py()) {
                            return Err(PyErr::fetch(obj.py()));
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

    #[test]
    fn from_complex() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let complex = Complex::new(3.0, 1.2);
        let py_c = PyComplex::from_complex(py, complex);
        assert_eq!(py_c.real(), 3.0);
        assert_eq!(py_c.imag(), 1.2);
    }
    #[test]
    fn to_from_complex() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let val = Complex::new(3.0, 1.2);
        let obj = val.to_object(py);
        assert_eq!(obj.extract::<Complex<f64>>(py).unwrap(), val);
    }
    #[test]
    fn from_complex_err() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let obj = vec![1].to_object(py);
        assert!(obj.extract::<Complex<f64>>(py).is_err());
    }
}

#[cfg(test)]
mod test {
    use super::PyComplex;
    use crate::python::Python;
    #[test]
    fn test_from_double() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let complex = PyComplex::from_doubles(py, 3.0, 1.2);
        assert_eq!(complex.real(), 3.0);
        assert_eq!(complex.imag(), 1.2);
    }

    #[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
    #[test]
    fn test_add() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let l = PyComplex::from_doubles(py, 3.0, 1.2);
        let r = PyComplex::from_doubles(py, 1.0, 2.6);
        let res = l + r;
        assert_approx_eq!(res.real(), 4.0);
        assert_approx_eq!(res.imag(), 3.8);
    }

    #[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
    #[test]
    fn test_sub() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let l = PyComplex::from_doubles(py, 3.0, 1.2);
        let r = PyComplex::from_doubles(py, 1.0, 2.6);
        let res = l - r;
        assert_approx_eq!(res.real(), 2.0);
        assert_approx_eq!(res.imag(), -1.4);
    }

    #[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
    #[test]
    fn test_mul() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let l = PyComplex::from_doubles(py, 3.0, 1.2);
        let r = PyComplex::from_doubles(py, 1.0, 2.6);
        let res = l * r;
        assert_approx_eq!(res.real(), -0.12);
        assert_approx_eq!(res.imag(), 9.0);
    }

    #[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
    #[test]
    fn test_div() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let l = PyComplex::from_doubles(py, 3.0, 1.2);
        let r = PyComplex::from_doubles(py, 1.0, 2.6);
        let res = l / r;
        assert_approx_eq!(res.real(), 0.7886597938144329);
        assert_approx_eq!(res.imag(), -0.8505154639175257);
    }

    #[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
    #[test]
    fn test_neg() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let val = PyComplex::from_doubles(py, 3.0, 1.2);
        let res = -val;
        assert_approx_eq!(res.real(), -3.0);
        assert_approx_eq!(res.imag(), -1.2);
    }

    #[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
    #[test]
    fn test_abs() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let val = PyComplex::from_doubles(py, 3.0, 1.2);
        assert_approx_eq!(val.abs(), 3.2310988842807022);
    }

    #[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
    #[test]
    fn test_pow() {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let l = PyComplex::from_doubles(py, 3.0, 1.2);
        let r = PyComplex::from_doubles(py, 1.2, 2.6);
        let val = l.pow(r);
        assert_approx_eq!(val.real(), -1.4193099970166037);
        assert_approx_eq!(val.imag(), -0.5412974660335446);
    }
}
