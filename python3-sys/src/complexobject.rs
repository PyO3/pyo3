use libc::{c_double, c_int};
use object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyComplex_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyComplex_Check(op : *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyComplex_Type)
}

#[inline(always)]
pub unsafe fn PyComplex_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyComplex_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyComplex_FromDoubles(real: c_double,
                                 imag: c_double) -> *mut PyObject;
    pub fn PyComplex_RealAsDouble(op: *mut PyObject) -> c_double;
    pub fn PyComplex_ImagAsDouble(op: *mut PyObject) -> c_double;
}

