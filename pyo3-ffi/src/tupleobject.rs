use crate::object::*;
use crate::pyport::Py_ssize_t;
use std::ffi::c_int;
use std::ptr::addr_of_mut;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyTuple_Type")]
    pub static mut PyTuple_Type: PyTypeObject;
    pub static mut PyTupleIter_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyTuple_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_TUPLE_SUBCLASS)
}

#[inline]
pub unsafe fn PyTuple_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyTuple_Type)) as c_int
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyTuple_New")]
    pub fn PyTuple_New(size: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyTuple_Size")]
    pub fn PyTuple_Size(arg1: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyTuple_GetItem")]
    pub fn PyTuple_GetItem(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyTuple_SetItem")]
    pub fn PyTuple_SetItem(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyTuple_GetSlice")]
    pub fn PyTuple_GetSlice(
        arg1: *mut PyObject,
        arg2: Py_ssize_t,
        arg3: Py_ssize_t,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyTuple_Pack")]
    pub fn PyTuple_Pack(arg1: Py_ssize_t, ...) -> *mut PyObject;
    #[cfg(not(Py_3_9))]
    pub fn PyTuple_ClearFreeList() -> c_int;
}
