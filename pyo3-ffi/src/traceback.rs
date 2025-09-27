use crate::object::*;
use std::ffi::c_int;
#[cfg(not(PyPy))]
use std::ptr::addr_of_mut;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyTraceBack_Here")]
    pub fn PyTraceBack_Here(arg1: *mut crate::PyFrameObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyTraceBack_Print")]
    pub fn PyTraceBack_Print(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyTraceBack_Type")]
    pub static mut PyTraceBack_Type: PyTypeObject;

    #[cfg(PyPy)]
    #[link_name = "PyPyTraceBack_Check"]
    pub fn PyTraceBack_Check(op: *mut PyObject) -> c_int;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyTraceBack_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyTraceBack_Type)) as c_int
}
