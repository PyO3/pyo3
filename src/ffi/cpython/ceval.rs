use crate::ffi::object::{freefunc, PyObject};
use crate::ffi::Py_tracefunc;

use std::os::raw::c_int;

extern "C" {
    pub fn PyEval_SetProfile(trace_func: Py_tracefunc, arg1: *mut PyObject);

    pub fn _PyEval_EvalFrameDefault(
        arg1: *mut crate::ffi::PyFrameObject,
        exc: c_int,
    ) -> *mut PyObject;
    pub fn _PyEval_RequestCodeExtraIndex(func: freefunc) -> c_int;

    pub fn PyEval_SetTrace(trace_func: Py_tracefunc, arg1: *mut PyObject);
}
