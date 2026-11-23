#[cfg(all(any(Py_3_10, not(Py_LIMITED_API)), any(not(PyPy), Py_3_12)))]
use crate::PyCodeObject;
use crate::PyFrameObject;
use core::ffi::c_int;

extern_libpython! {
    #[cfg_attr(all(PyPy, not(Py_3_12)), link_name = "PyPyFrame_GetLineNumber")]
    pub fn PyFrame_GetLineNumber(frame: *mut PyFrameObject) -> c_int;

    #[cfg(all(any(Py_3_10, not(Py_LIMITED_API)), any(not(PyPy), Py_3_12)))]
    pub fn PyFrame_GetCode(frame: *mut PyFrameObject) -> *mut PyCodeObject;
}
