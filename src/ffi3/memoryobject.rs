use crate::ffi3::object::*;
use crate::ffi3::pyport::Py_ssize_t;
use std::os::raw::{c_char, c_int};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyMemoryView_Type")]
    pub static mut PyMemoryView_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyMemoryView_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyMemoryView_Type) as c_int
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyMemoryView_FromObject")]
    pub fn PyMemoryView_FromObject(base: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyMemoryView_FromMemory")]
    pub fn PyMemoryView_FromMemory(
        mem: *mut c_char,
        size: Py_ssize_t,
        flags: c_int,
    ) -> *mut PyObject;
    pub fn PyMemoryView_GetContiguous(
        base: *mut PyObject,
        buffertype: c_int,
        order: c_char,
    ) -> *mut PyObject;
}
