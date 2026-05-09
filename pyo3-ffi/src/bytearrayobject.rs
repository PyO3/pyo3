use crate::object::*;
use crate::pyport::Py_ssize_t;
use std::ffi::{c_char, c_int};

#[cfg(not(RustPython))]
extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyByteArray_Type")]
    pub static mut PyByteArray_Type: PyTypeObject;

    pub static mut PyByteArrayIter_Type: PyTypeObject;
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyByteArray_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &raw mut PyByteArray_Type)
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyByteArray_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyByteArray_Type)
}

extern_libpython! {
    #[cfg(RustPython)]
    pub fn PyByteArray_Check(op: *mut PyObject) -> c_int;
    #[cfg(RustPython)]
    pub fn PyByteArray_CheckExact(op: *mut PyObject) -> c_int;

    #[cfg_attr(PyPy, link_name = "PyPyByteArray_FromObject")]
    pub fn PyByteArray_FromObject(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyByteArray_Concat")]
    pub fn PyByteArray_Concat(a: *mut PyObject, b: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyByteArray_FromStringAndSize")]
    pub fn PyByteArray_FromStringAndSize(string: *const c_char, len: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyByteArray_Size")]
    pub fn PyByteArray_Size(bytearray: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyByteArray_AsString")]
    pub fn PyByteArray_AsString(bytearray: *mut PyObject) -> *mut c_char;
    #[cfg_attr(PyPy, link_name = "PyPyByteArray_Resize")]
    pub fn PyByteArray_Resize(bytearray: *mut PyObject, len: Py_ssize_t) -> c_int;
}
