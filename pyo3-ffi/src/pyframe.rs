#[allow(unused_imports)]
use crate::object::PyObject;
#[cfg(not(GraalPy))]
#[cfg(any(Py_3_10, all(Py_3_9, not(Py_LIMITED_API))))]
use crate::PyCodeObject;
use crate::PyFrameObject;
use std::ffi::c_int;

extern "C" {
    pub fn PyFrame_GetLineNumber(frame: *mut PyFrameObject) -> c_int;

    #[cfg(not(GraalPy))]
    #[cfg(any(Py_3_10, all(Py_3_9, not(Py_LIMITED_API))))]
    pub fn PyFrame_GetCode(frame: *mut PyFrameObject) -> *mut PyCodeObject;
}
