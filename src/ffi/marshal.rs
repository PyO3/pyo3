use super::PyObject;
use std::os::raw::{c_char, c_int};

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyMarshal_WriteObjectToString")]
    pub fn PyMarshal_WriteObjectToString(object: *mut PyObject, version: c_int) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyMarshal_ReadObjectFromString")]
    pub fn PyMarshal_ReadObjectFromString(data: *const c_char, len: isize) -> *mut PyObject;
}
