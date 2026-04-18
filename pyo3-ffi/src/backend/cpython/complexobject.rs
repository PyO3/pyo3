use crate::object::*;
use std::ffi::{c_double, c_int};
use std::ptr::addr_of_mut;

opaque_struct!(pub PyComplexObject);

#[repr(C)]
pub struct Py_complex {
    pub real: c_double,
    pub imag: c_double,
}

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyComplex_Type")]
    pub static mut PyComplex_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyComplex_AsCComplex")]
    pub fn PyComplex_AsCComplex(op: *mut PyObject) -> Py_complex;
}

#[inline]
pub unsafe fn PyComplex_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut!(PyComplex_Type))
}

#[inline]
pub unsafe fn PyComplex_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, addr_of_mut!(PyComplex_Type))
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
