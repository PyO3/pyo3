use crate::ffi3::object::*;
use crate::ffi3::pyport::Py_ssize_t;
use std::os::raw::c_int;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyListObject {
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub ob_size: Py_ssize_t,
    pub ob_item: *mut *mut PyObject,
    pub allocated: Py_ssize_t,
}

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
    (Py_TYPE(op) == &mut PyList_Type) as c_int
}

/// Macro, trading safety for speed
#[cfg(not(Py_LIMITED_API))]
#[inline]
pub unsafe fn PyList_GET_ITEM(op: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
    *(*(op as *mut PyListObject)).ob_item.offset(i as isize)
}

#[cfg(not(Py_LIMITED_API))]
#[inline]
pub unsafe fn PyList_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    Py_SIZE(op)
}

/// Macro, *only* to be used to fill in brand new lists
#[cfg(not(Py_LIMITED_API))]
#[inline]
pub unsafe fn PyList_SET_ITEM(op: *mut PyObject, i: Py_ssize_t, v: *mut PyObject) {
    *(*(op as *mut PyListObject)).ob_item.offset(i as isize) = v;
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyList_New")]
    pub fn PyList_New(size: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyList_Size")]
    pub fn PyList_Size(arg1: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyList_GetItem")]
    pub fn PyList_GetItem(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject;
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
    #[cfg_attr(PyPy, link_name = "PyPyList_Sort")]
    pub fn PyList_Sort(arg1: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyList_Reverse")]
    pub fn PyList_Reverse(arg1: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyList_AsTuple")]
    pub fn PyList_AsTuple(arg1: *mut PyObject) -> *mut PyObject;
}
