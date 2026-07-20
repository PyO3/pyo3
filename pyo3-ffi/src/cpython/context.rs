use crate::object::PyObject;
use crate::object::PyTypeObject;
use crate::Py_IS_TYPE;
#[cfg(all(Py_3_14, not(any(PyPy, GraalPy))))]
use core::ffi::c_uint;
use core::ffi::{c_char, c_int};

extern_libpython! {
    pub static mut PyContext_Type: PyTypeObject;
    // skipped non-limited opaque PyContext
    pub static mut PyContextVar_Type: PyTypeObject;
    // skipped non-limited opaque PyContextVar
    pub static mut PyContextToken_Type: PyTypeObject;
    // skipped non-limited opaque PyContextToken
}

#[inline]
pub unsafe fn PyContext_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyContext_Type)
}

#[inline]
pub unsafe fn PyContextVar_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyContextVar_Type)
}

#[inline]
pub unsafe fn PyContextToken_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyContextToken_Type)
}

extern_libpython! {
    pub fn PyContext_New() -> *mut PyObject;
    pub fn PyContext_Copy(ctx: *mut PyObject) -> *mut PyObject;
    pub fn PyContext_CopyCurrent() -> *mut PyObject;

    pub fn PyContext_Enter(ctx: *mut PyObject) -> c_int;
    pub fn PyContext_Exit(ctx: *mut PyObject) -> c_int;
}

// Use the C enum's integer representation to permit future event values.
#[cfg(all(Py_3_14, not(any(PyPy, GraalPy))))]
pub type PyContextEvent = c_uint;

#[cfg(all(Py_3_14, not(any(PyPy, GraalPy))))]
pub const Py_CONTEXT_SWITCHED: PyContextEvent = 1;

#[cfg(all(Py_3_14, not(any(PyPy, GraalPy))))]
pub type PyContext_WatchCallback =
    unsafe extern "C" fn(event: PyContextEvent, obj: *mut PyObject) -> c_int;

extern_libpython! {
    #[cfg(all(Py_3_14, not(any(PyPy, GraalPy))))]
    pub fn PyContext_AddWatcher(callback: PyContext_WatchCallback) -> c_int;
    #[cfg(all(Py_3_14, not(any(PyPy, GraalPy))))]
    pub fn PyContext_ClearWatcher(watcher_id: c_int) -> c_int;

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
