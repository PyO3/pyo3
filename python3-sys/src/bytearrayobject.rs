use libc::{c_char, c_int};
use object::*;
use pyport::Py_ssize_t;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyByteArray_Type: PyTypeObject;
    pub static mut PyByteArrayIter_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyByteArray_Check(op : *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyByteArray_Type)
}

#[inline(always)]
pub unsafe fn PyByteArray_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyByteArray_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyByteArray_FromObject(o: *mut PyObject) -> *mut PyObject;
    pub fn PyByteArray_Concat(a: *mut PyObject, b: *mut PyObject)
     -> *mut PyObject;
    pub fn PyByteArray_FromStringAndSize(string: *const c_char,
                                         len: Py_ssize_t) -> *mut PyObject;
    pub fn PyByteArray_Size(bytearray: *mut PyObject) -> Py_ssize_t;
    pub fn PyByteArray_AsString(bytearray: *mut PyObject) -> *mut c_char;
    pub fn PyByteArray_Resize(bytearray: *mut PyObject, len: Py_ssize_t)
     -> c_int;
}

