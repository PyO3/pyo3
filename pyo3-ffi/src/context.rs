use crate::object::{PyObject, PyTypeObject, Py_TYPE};
use std::ffi::{c_char, c_int};
use std::ptr::addr_of_mut;

extern "C" {
    pub static mut PyContext_Type: PyTypeObject;
    // skipped non-limited opaque PyContext
    pub static mut PyContextVar_Type: PyTypeObject;
    // skipped non-limited opaque PyContextVar
    pub static mut PyContextToken_Type: PyTypeObject;
    // skipped non-limited opaque PyContextToken
}

#[inline]
pub unsafe fn PyContext_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyContext_Type)) as c_int
}

#[inline]
pub unsafe fn PyContextVar_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyContextVar_Type)) as c_int
}

#[inline]
pub unsafe fn PyContextToken_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyContextToken_Type)) as c_int
}

extern "C" {
    pub fn PyContext_New() -> *mut PyObject;
    pub fn PyContext_Copy(ctx: *mut PyObject) -> *mut PyObject;
    pub fn PyContext_CopyCurrent() -> *mut PyObject;

    pub fn PyContext_Enter(ctx: *mut PyObject) -> c_int;
    pub fn PyContext_Exit(ctx: *mut PyObject) -> c_int;

    pub fn PyContextVar_New(name: *const c_char, def: *mut PyObject) -> *mut PyObject;
    pub fn PyContextVar_Get(
        var: *mut PyObject,
        default_value: *mut PyObject,
        value: *mut *mut PyObject,
    ) -> c_int;
    pub fn PyContextVar_Set(var: *mut PyObject, value: *mut PyObject) -> *mut PyObject;
    pub fn PyContextVar_Reset(var: *mut PyObject, token: *mut PyObject) -> c_int;
    // skipped non-limited _PyContext_NewHamtForTests
}
