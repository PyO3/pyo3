use crate::ffi3::object::*;
use std::os::raw::c_int;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyTraceBack_Here")]
    pub fn PyTraceBack_Here(arg1: *mut crate::ffi3::PyFrameObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyTraceBack_Print")]
    pub fn PyTraceBack_Print(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyTraceBack_Type")]
    pub static mut PyTraceBack_Type: PyTypeObject;
}

#[inline]
#[cfg_attr(PyPy, link_name = "PyPyTraceBack_Check")]
pub unsafe fn PyTraceBack_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyTraceBack_Type) as c_int
}
