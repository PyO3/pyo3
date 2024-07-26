use crate::cpython::pystate::Py_tracefunc;
use crate::object::{freefunc, PyObject};
use std::os::raw::c_int;

extern "C" {
    pub fn PyEval_SetProfile(trace_func: Option<Py_tracefunc>, arg1: *mut PyObject);
    #[cfg(Py_3_12)]
    pub fn PyEval_SetProfileAllThreads(trace_func: Option<Py_tracefunc>, arg1: *mut PyObject);
    pub fn PyEval_SetTrace(trace_func: Option<Py_tracefunc>, arg1: *mut PyObject);
    pub fn PyEval_SetTraceAllThreads(trace_func: Option<Py_tracefunc>, arg1: *mut PyObject);

    // skipped PyEval_MergeCompilerFlags

    #[cfg(not(Py_3_11))]
    pub fn _PyEval_EvalFrameDefault(arg1: *mut crate::PyFrameObject, exc: c_int) -> *mut PyObject;

    #[cfg(Py_3_11)]
    pub fn _PyEval_EvalFrameDefault(
        tstate: *mut crate::PyThreadState,
        frame: *mut crate::_PyInterpreterFrame,
        exc: c_int,
    ) -> *mut crate::PyObject;

    // skipped PyUnstable_Eval_RequestCodeExtraIndex
    #[cfg(not(Py_3_13))]
    pub fn _PyEval_RequestCodeExtraIndex(func: freefunc) -> c_int;

    // skipped private _PyEval_SliceIndex
    // skipped private _PyEval_SliceIndexNotNone
}
