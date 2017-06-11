use std::os::raw::{c_int, c_char};
use ffi3::pyport::Py_ssize_t;
use ffi3::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyMemoryView_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyMemoryView_Check(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyMemoryView_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyMemoryView_FromObject(base: *mut PyObject) -> *mut PyObject;
    pub fn PyMemoryView_FromMemory(mem: *mut c_char, size: Py_ssize_t,
                                   flags: c_int) -> *mut PyObject;
    pub fn PyMemoryView_GetContiguous(base: *mut PyObject,
                                      buffertype: c_int,
                                      order: c_char) -> *mut PyObject;
}

