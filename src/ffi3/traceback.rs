use std::os::raw::c_int;
use ffi3::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyTraceBack_Here(arg1: *mut ::ffi3::PyFrameObject) -> c_int;
    pub fn PyTraceBack_Print(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;
    pub static mut PyTraceBack_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyTraceBack_Check(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyTraceBack_Type) as c_int
}

