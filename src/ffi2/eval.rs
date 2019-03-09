use std::os::raw::c_int;
use ffi2::object::PyObject;
use ffi2::code::PyCodeObject;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyEval_EvalCode(arg1: *mut PyCodeObject, arg2: *mut PyObject,
                           arg3: *mut PyObject) -> *mut PyObject;
    pub fn PyEval_EvalCodeEx(co: *mut PyCodeObject, globals: *mut PyObject,
                             locals: *mut PyObject, args: *mut *mut PyObject,
                             argc: c_int, kwds: *mut *mut PyObject,
                             kwdc: c_int, defs: *mut *mut PyObject,
                             defc: c_int, closure: *mut PyObject)
                             -> *mut PyObject;
    fn _PyEval_CallTracing(func: *mut PyObject, args: *mut PyObject)
                           -> *mut PyObject;
}
