use std::os::raw::{c_int, c_double};
use ffi3::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyFloat_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyFloat_Check(op : *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyFloat_Type)
}

#[inline(always)]
pub unsafe fn PyFloat_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyFloat_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyFloat_GetMax() -> c_double;
    pub fn PyFloat_GetMin() -> c_double;
    pub fn PyFloat_GetInfo() -> *mut PyObject;
    pub fn PyFloat_FromString(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyFloat_FromDouble(arg1: c_double) -> *mut PyObject;
    pub fn PyFloat_AsDouble(arg1: *mut PyObject) -> c_double;
}

