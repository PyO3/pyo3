use ffi;
use instance::PyObjectWithToken;
use object::PyObject;
use python::{Python, ToPyPointer};
#[cfg(any(not(Py_LIMITED_API), not(Py_3)))]
use std::ops::*;
use std::os::raw::c_double;

/// Represents a Python `complex`.
#[repr(transparent)]
pub struct PyComplex(PyObject);

pyobject_native_type!(PyComplex, ffi::PyComplex_Type, ffi::PyComplex_Check);

impl PyComplex {
    pub fn from_doubles<'p>(py: Python<'p>, real: c_double, imag: c_double) -> &'p PyComplex {
        unsafe {
            let ptr = ffi::PyComplex_FromDoubles(real, imag);
            py.from_owned_ptr(ptr)
        }
    }
    pub fn real(&self) -> c_double {
        unsafe { ffi::PyComplex_RealAsDouble(self.as_ptr()) }
    }
    pub fn imag(&self) -> c_double {
        unsafe { ffi::PyComplex_ImagAsDouble(self.as_ptr()) }
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

#[cfg(test)]
mod test {
    use super::PyComplex;
    use python::Python;
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
}
