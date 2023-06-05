use crate::object::*;
use std::os::raw::{c_double, c_int};
use std::ptr::addr_of_mut;

#[repr(C)]
#[derive(Copy, Clone)]
// non-limited
pub struct Py_complex {
    pub real: c_double,
    pub imag: c_double,
}

#[cfg(not(Py_LIMITED_API))]
extern "C" {
    pub fn _Py_c_sum(left: Py_complex, right: Py_complex) -> Py_complex;
    pub fn _Py_c_diff(left: Py_complex, right: Py_complex) -> Py_complex;
    pub fn _Py_c_neg(complex: Py_complex) -> Py_complex;
    pub fn _Py_c_prod(left: Py_complex, right: Py_complex) -> Py_complex;
    pub fn _Py_c_quot(dividend: Py_complex, divisor: Py_complex) -> Py_complex;
    pub fn _Py_c_pow(num: Py_complex, exp: Py_complex) -> Py_complex;
    pub fn _Py_c_abs(arg: Py_complex) -> c_double;
    #[cfg_attr(PyPy, link_name = "PyPyComplex_FromCComplex")]
    pub fn PyComplex_FromCComplex(v: Py_complex) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyComplex_AsCComplex")]
    pub fn PyComplex_AsCComplex(op: *mut PyObject) -> Py_complex;
}

#[repr(C)]
#[derive(Copy, Clone)]
// non-limited
pub struct PyComplexObject {
    pub ob_base: PyObject,
    pub cval: Py_complex,
}

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
    (Py_TYPE(op) == addr_of_mut!(PyComplex_Type)) as c_int
}

extern "C" {
    // skipped non-limited PyComplex_FromCComplex
    #[cfg_attr(PyPy, link_name = "PyPyComplex_FromDoubles")]
    pub fn PyComplex_FromDoubles(real: c_double, imag: c_double) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyComplex_RealAsDouble")]
    pub fn PyComplex_RealAsDouble(op: *mut PyObject) -> c_double;
    #[cfg_attr(PyPy, link_name = "PyPyComplex_ImagAsDouble")]
    pub fn PyComplex_ImagAsDouble(op: *mut PyObject) -> c_double;
    // skipped non-limited PyComplex_AsCComplex
    // skipped non-limited _PyComplex_FormatAdvancedWriter
}
