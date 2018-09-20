use ffi;
use object::PyObject;
use python::{Python, ToPyPointer};
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
}
