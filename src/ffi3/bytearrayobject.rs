use std::os::raw::{c_char, c_int};
use ffi3::object::*;
use ffi3::pyport::Py_ssize_t;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="PyPyByteArray_Type")]
    pub static mut PyByteArray_Type: PyTypeObject;
    pub static mut PyByteArrayIter_Type: PyTypeObject;
}

#[inline(always)]
#[cfg_attr(PyPy, link_name="PyPyByteArray_Check")]
pub unsafe fn PyByteArray_Check(op : *mut PyObject) -> c_int {
    #[cfg_attr(PyPy, link_name="PyPyObject_Type")]
    PyObject_TypeCheck(op, &mut PyByteArray_Type)
}

#[inline(always)]
#[cfg_attr(PyPy, link_name="PyPyByteArray_CheckExact")]
pub unsafe fn PyByteArray_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyByteArray_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="PyPyByteArray_FromObject")]
    pub fn PyByteArray_FromObject(o: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="PyPyByteArray_Concat")]
    pub fn PyByteArray_Concat(a: *mut PyObject, b: *mut PyObject)
     -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="PyPyByteArray_FromStringAndSize")]
    pub fn PyByteArray_FromStringAndSize(string: *const c_char,
                                         len: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="PyPyByteArray_Size")]
    pub fn PyByteArray_Size(bytearray: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name="PyPyByteArray_AsString")]
    pub fn PyByteArray_AsString(bytearray: *mut PyObject) -> *mut c_char;
    #[cfg_attr(PyPy, link_name="PyPyByteArray_Resize")]
    pub fn PyByteArray_Resize(bytearray: *mut PyObject, len: Py_ssize_t)
     -> c_int;
}