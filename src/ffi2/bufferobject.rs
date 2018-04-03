use std::os::raw::{c_void, c_int};
use ffi2::object::*;
use ffi2::pyport::Py_ssize_t;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyBuffer_Type")]
    pub static mut PyBuffer_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyBuffer_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyBuffer_Type;
    (Py_TYPE(op) == u) as c_int
}

pub const Py_END_OF_BUFFER: Py_ssize_t = -1;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyBuffer_FromObject")]
    pub fn PyBuffer_FromObject(base: *mut PyObject, offset: Py_ssize_t,
                               size: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyBuffer_FromReadWriteObject")]
    pub fn PyBuffer_FromReadWriteObject(base: *mut PyObject,
                                        offset: Py_ssize_t, size: Py_ssize_t)
                                        -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyBuffer_FromMemory")]
    pub fn PyBuffer_FromMemory(ptr: *mut c_void, size: Py_ssize_t)
                               -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyBuffer_FromReadWriteMemory")]
    pub fn PyBuffer_FromReadWriteMemory(ptr: *mut c_void,
                                        size: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPyBuffer_New")]
    pub fn PyBuffer_New(size: Py_ssize_t) -> *mut PyObject;
}