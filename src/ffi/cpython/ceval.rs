#[cfg(not(Py_LIMITED_API))]
use crate::ffi::object::FreeFunc;
use crate::ffi::object::PyObject;
use crate::ffi::pystate::Py_tracefunc;
use std::os::raw::c_int;

extern "C" {
    pub fn _PyEval_EvalFrameDefault(
        arg1: *mut crate::ffi::PyFrameObject,
        exc: c_int,
    ) -> *mut PyObject;
    #[cfg(not(Py_LIMITED_API))]
    pub fn _PyEval_RequestCodeExtraIndex(func: FreeFunc) -> c_int;
    pub fn PyEval_SetProfile(trace_func: Py_tracefunc, arg1: *mut PyObject);
    pub fn PyEval_SetTrace(trace_func: Py_tracefunc, arg1: *mut PyObject);
}
