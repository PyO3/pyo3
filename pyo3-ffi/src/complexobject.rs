use crate::object::*;
use std::ffi::{c_double, c_int};

#[cfg(not(RustPython))]
extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyComplex_Type")]
    pub static mut PyComplex_Type: PyTypeObject;
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyComplex_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &raw mut PyComplex_Type)
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyComplex_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyComplex_Type)
}

extern_libpython! {
    #[cfg(RustPython)]
    pub fn PyComplex_Check(op: *mut PyObject) -> c_int;
    #[cfg(RustPython)]
    pub fn PyComplex_CheckExact(op: *mut PyObject) -> c_int;
}

extern_libpython! {
    // skipped non-limited PyComplex_FromCComplex
    #[cfg_attr(PyPy, link_name = "PyPyComplex_FromDoubles")]
    pub fn PyComplex_FromDoubles(real: c_double, imag: c_double) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyComplex_RealAsDouble")]
    pub fn PyComplex_RealAsDouble(op: *mut PyObject) -> c_double;
    #[cfg_attr(PyPy, link_name = "PyPyComplex_ImagAsDouble")]
    pub fn PyComplex_ImagAsDouble(op: *mut PyObject) -> c_double;
}
