use crate::object::*;
use std::ffi::{c_double, c_int};
use std::ptr::addr_of_mut;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyComplex_Type")]
    pub static mut PyComplex_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyComplex_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut!(PyComplex_Type))
}

#[inline]
pub unsafe fn PyComplex_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, addr_of_mut!(PyComplex_Type))
}

extern "C" {
    // skipped non-limited PyComplex_FromCComplex
    #[cfg_attr(PyPy, link_name = "PyPyComplex_FromDoubles")]
    pub fn PyComplex_FromDoubles(real: c_double, imag: c_double) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyComplex_RealAsDouble")]
    pub fn PyComplex_RealAsDouble(op: *mut PyObject) -> c_double;
    #[cfg_attr(PyPy, link_name = "PyPyComplex_ImagAsDouble")]
    pub fn PyComplex_ImagAsDouble(op: *mut PyObject) -> c_double;
}
