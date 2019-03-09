use std::os::raw::c_int;
use ffi2::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PySeqIter_Type: PyTypeObject;
    pub static mut PyCallIter_Type: PyTypeObject;

    pub fn PySeqIter_New(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyCallIter_New(arg1: *mut PyObject, arg2: *mut PyObject)
                          -> *mut PyObject;
}

#[inline(always)]
pub unsafe fn PySeqIter_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PySeqIter_Type) as c_int
}

#[inline(always)]
pub unsafe fn PyCallIter_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyCallIter_Type) as c_int
}
