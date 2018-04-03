use std::os::raw::{c_double, c_int};
use ffi3::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyComplex_Type")]
    pub static mut PyComplex_Type: PyTypeObject;
}

#[inline(always)]
#[cfg_attr(PyPy, link_name="\u{1}_PyPyComplex_Check")]
pub unsafe fn PyComplex_Check(op : *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyComplex_Type)
}

#[inline(always)]
#[cfg_attr(PyPy, link_name="\u{1}_PyPyComplex_CheckExact")]
pub unsafe fn PyComplex_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyComplex_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyComplex_FromDoubles")]
    pub fn PyComplex_FromDoubles(real: c_double,
                                 imag: c_double) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyComplex_RealAsDouble")]
    pub fn PyComplex_RealAsDouble(op: *mut PyObject) -> c_double;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyComplex_ImagAsDouble")]
    pub fn PyComplex_ImagAsDouble(op: *mut PyObject) -> c_double;
}