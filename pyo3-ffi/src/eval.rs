use crate::object::PyObject;
use std::os::raw::c_int;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyEval_EvalCode")]
    pub fn PyEval_EvalCode(
        arg1: *mut PyObject,
        arg2: *mut PyObject,
        arg3: *mut PyObject,
    ) -> *mut PyObject;
    pub fn PyEval_EvalCodeEx(
        co: *mut PyObject,
        globals: *mut PyObject,
        locals: *mut PyObject,
        args: *mut *mut PyObject,
        argc: c_int,
        kwds: *mut *mut PyObject,
        kwdc: c_int,
        defs: *mut *mut PyObject,
        defc: c_int,
        kwdefs: *mut PyObject,
        closure: *mut PyObject,
    ) -> *mut PyObject;

    // skipped non-limited _PyEval_EvalCodeWithName
    // skipped non-limited _PyEval_CallTracing
}
