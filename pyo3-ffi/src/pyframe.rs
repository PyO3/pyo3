#[cfg(Py_3_9)]
use crate::PyCodeObject;
#[cfg(not(Py_LIMITED_API))]
use crate::PyFrameObject;
use std::os::raw::c_int;

#[cfg(Py_LIMITED_API)]
opaque_struct!(PyFrameObject);

extern "C" {
    pub fn PyFrame_GetLineNumber(f: *mut PyFrameObject) -> c_int;
    #[cfg(Py_3_9)]
    pub fn PyFrame_GetCode(f: *mut PyFrameObject) -> *mut PyCodeObject;
}
