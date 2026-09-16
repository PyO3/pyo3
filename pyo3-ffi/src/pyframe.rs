#[cfg(not(GraalPy))]
#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
use crate::PyCodeObject;
use crate::PyFrameObject;
use core::ffi::c_int;

extern_libpython! {
    #[cfg_attr(all(PyPy, not(Py_3_12)), link_name = "PyPyFrame_GetLineNumber")]
    pub fn PyFrame_GetLineNumber(frame: *mut PyFrameObject) -> c_int;

    #[cfg(not(GraalPy))]
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    pub fn PyFrame_GetCode(frame: *mut PyFrameObject) -> *mut PyCodeObject;
}
