use crate::ffi3::object::*;
use crate::ffi3::pyport::Py_ssize_t;
use std::os::raw::c_int;

#[repr(C)]
#[cfg(not(Py_LIMITED_API))]
pub struct PyTupleObject {
    pub ob_base: PyVarObject,
    pub ob_item: [*mut PyObject; 1],
}

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
    (Py_TYPE(op) == &mut PyTuple_Type) as c_int
}

#[cfg_attr(windows, link(name = "pythonXY"))]
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
    pub fn PyTuple_ClearFreeList() -> c_int;
}

/// Macro, trading safety for speed
#[inline]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn PyTuple_GET_ITEM(op: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
    *(*(op as *mut PyTupleObject))
        .ob_item
        .as_ptr()
        .offset(i as isize)
}

#[inline]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn PyTuple_GET_SIZE(op: *mut PyObject) -> Py_ssize_t {
    Py_SIZE(op)
}

/// Macro, *only* to be used to fill in brand new tuples
#[inline]
#[cfg(not(Py_LIMITED_API))]
pub unsafe fn PyTuple_SET_ITEM(op: *mut PyObject, i: Py_ssize_t, v: *mut PyObject) {
    *(*(op as *mut PyTupleObject))
        .ob_item
        .as_mut_ptr()
        .offset(i as isize) = v;
}
