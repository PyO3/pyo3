use crate::object::*;
use std::ffi::c_int;

#[cfg(not(RustPython))]
extern_libpython! {
    pub static mut PySeqIter_Type: PyTypeObject;
    pub static mut PyCallIter_Type: PyTypeObject;
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PySeqIter_Check(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PySeqIter_Type)
}

extern_libpython! {
    #[cfg(RustPython)]
    pub fn PySeqIter_Check(op: *mut PyObject) -> c_int;
}

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPySeqIter_New")]
    pub fn PySeqIter_New(arg1: *mut PyObject) -> *mut PyObject;
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyCallIter_Check(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyCallIter_Type)
}

extern_libpython! {
    #[cfg(RustPython)]
    pub fn PyCallIter_Check(op: *mut PyObject) -> c_int;
}

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyCallIter_New")]
    pub fn PyCallIter_New(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
}
