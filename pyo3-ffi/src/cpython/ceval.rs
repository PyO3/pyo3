use crate::cpython::pystate::Py_tracefunc;
use crate::object::{freefunc, PyObject};
use std::ffi::c_int;

extern "C" {
    // skipped non-limited _PyEval_CallTracing

    #[cfg(not(Py_3_11))]
    pub fn _PyEval_EvalFrameDefault(arg1: *mut crate::PyFrameObject, exc: c_int) -> *mut PyObject;

    #[cfg(Py_3_11)]
    pub fn _PyEval_EvalFrameDefault(
        tstate: *mut crate::PyThreadState,
        frame: *mut crate::_PyInterpreterFrame,
        exc: c_int,
    ) -> *mut crate::PyObject;

    pub fn _PyEval_RequestCodeExtraIndex(func: freefunc) -> c_int;
    pub fn PyEval_SetProfile(trace_func: Option<Py_tracefunc>, arg1: *mut PyObject);
    pub fn PyEval_SetTrace(trace_func: Option<Py_tracefunc>, arg1: *mut PyObject);
}
