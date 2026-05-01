use crate::object::PyObject;
#[cfg(not(RustPython))]
use crate::object::PyTypeObject;
#[cfg(not(RustPython))]
use crate::Py_IS_TYPE;
use std::ffi::{c_char, c_int};

#[cfg(not(RustPython))]
extern_libpython! {
    pub static mut PyContext_Type: PyTypeObject;
    // skipped non-limited opaque PyContext
    pub static mut PyContextVar_Type: PyTypeObject;
    // skipped non-limited opaque PyContextVar
    pub static mut PyContextToken_Type: PyTypeObject;
    // skipped non-limited opaque PyContextToken
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyContext_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyContext_Type)
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyContextVar_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyContextVar_Type)
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyContextToken_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyContextToken_Type)
}

extern_libpython! {
    #[cfg(RustPython)]
    pub fn PyContext_CheckExact(op: *mut PyObject) -> c_int;
    #[cfg(RustPython)]
    pub fn PyContextVar_CheckExact(op: *mut PyObject) -> c_int;
    #[cfg(RustPython)]
    pub fn PyContextToken_CheckExact(op: *mut PyObject) -> c_int;

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
