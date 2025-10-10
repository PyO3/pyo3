use crate::object::*;
use std::ffi::{c_double, c_int};
use std::ptr::addr_of_mut;

#[cfg(Py_LIMITED_API)]
// TODO: remove (see https://github.com/PyO3/pyo3/pull/1341#issuecomment-751515985)
opaque_struct!(pub PyFloatObject);

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyFloat_Type")]
    pub static mut PyFloat_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyFloat_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut!(PyFloat_Type))
}

#[inline]
pub unsafe fn PyFloat_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyFloat_Type)) as c_int
}

// skipped Py_RETURN_NAN
// skipped Py_RETURN_INF

extern "C" {
    pub fn PyFloat_GetMax() -> c_double;
    pub fn PyFloat_GetMin() -> c_double;
    pub fn PyFloat_GetInfo() -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyFloat_FromString")]
    pub fn PyFloat_FromString(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyFloat_FromDouble")]
    pub fn PyFloat_FromDouble(arg1: c_double) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyFloat_AsDouble")]
    pub fn PyFloat_AsDouble(arg1: *mut PyObject) -> c_double;
}

// skipped non-limited _PyFloat_Pack2
// skipped non-limited _PyFloat_Pack4
// skipped non-limited _PyFloat_Pack8
// skipped non-limited _PyFloat_Unpack2
// skipped non-limited _PyFloat_Unpack4
// skipped non-limited _PyFloat_Unpack8
// skipped non-limited _PyFloat_DebugMallocStats
// skipped non-limited _PyFloat_FormatAdvancedWriter
