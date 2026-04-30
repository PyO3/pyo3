use crate::object::*;
use std::ffi::c_int;

#[cfg(not(any(PyPy, GraalPy)))]
#[repr(C)]
pub struct PyCellObject {
    pub ob_base: PyObject,
    pub ob_ref: *mut PyObject,
}

extern_libpython! {
    pub fn PyCell_New(o: *mut PyObject) -> *mut PyObject;
    pub fn PyCell_Get(o: *mut PyObject) -> *mut PyObject;
    pub fn PyCell_Set(o: *mut PyObject, val: *mut PyObject) -> *mut PyObject;
    pub static mut PyCell_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyCell_Check(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyCell_Type)
}

// skipped PyCell_SET
// skipped PyCell_GET
