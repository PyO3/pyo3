use crate::object::*;
use crate::pyport::Py_ssize_t;
use std::ffi::c_int;
use std::ptr::addr_of_mut;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyList_Type")]
    pub static mut PyList_Type: PyTypeObject;
    pub static mut PyListIter_Type: PyTypeObject;
    pub static mut PyListRevIter_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyList_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_LIST_SUBCLASS)
}

#[inline]
pub unsafe fn PyList_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyList_Type)) as c_int
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyList_New")]
    pub fn PyList_New(size: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyList_Size")]
    pub fn PyList_Size(arg1: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyList_GetItem")]
    pub fn PyList_GetItem(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject;
    #[cfg(Py_3_13)]
    #[cfg_attr(PyPy, link_name = "PyPyList_GetItemRef")]
    pub fn PyList_GetItemRef(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyList_SetItem")]
    pub fn PyList_SetItem(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyList_Insert")]
    pub fn PyList_Insert(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyList_Append")]
    pub fn PyList_Append(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyList_GetSlice")]
    pub fn PyList_GetSlice(
        arg1: *mut PyObject,
        arg2: Py_ssize_t,
        arg3: Py_ssize_t,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyList_SetSlice")]
    pub fn PyList_SetSlice(
        arg1: *mut PyObject,
        arg2: Py_ssize_t,
        arg3: Py_ssize_t,
        arg4: *mut PyObject,
    ) -> c_int;
    #[cfg(Py_3_13)]
    pub fn PyList_Extend(list: *mut PyObject, iterable: *mut PyObject) -> c_int;
    #[cfg(Py_3_13)]
    pub fn PyList_Clear(list: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyList_Sort")]
    pub fn PyList_Sort(arg1: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyList_Reverse")]
    pub fn PyList_Reverse(arg1: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyList_AsTuple")]
    pub fn PyList_AsTuple(arg1: *mut PyObject) -> *mut PyObject;

    // CPython macros exported as functions on PyPy or GraalPy
    #[cfg(any(PyPy, GraalPy))]
    #[cfg_attr(PyPy, link_name = "PyPyList_GET_ITEM")]
    #[cfg_attr(GraalPy, link_name = "PyList_GetItem")]
    pub fn PyList_GET_ITEM(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject;
    #[cfg(PyPy)]
    #[cfg_attr(PyPy, link_name = "PyPyList_GET_SIZE")]
    pub fn PyList_GET_SIZE(arg1: *mut PyObject) -> Py_ssize_t;
    #[cfg(any(PyPy, GraalPy))]
    #[cfg_attr(PyPy, link_name = "PyPyList_SET_ITEM")]
    #[cfg_attr(GraalPy, link_name = "_PyList_SET_ITEM")]
    pub fn PyList_SET_ITEM(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: *mut PyObject);
}
