use crate::object::*;
use std::ffi::c_int;

extern_libpython! {
    pub static mut PySeqIter_Type: PyTypeObject;
    pub static mut PyCallIter_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PySeqIter_Check(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PySeqIter_Type)
}

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPySeqIter_New")]
    pub fn PySeqIter_New(arg1: *mut PyObject) -> *mut PyObject;
}

#[inline]
pub unsafe fn PyCallIter_Check(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyCallIter_Type)
}

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyCallIter_New")]
    pub fn PyCallIter_New(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
}
