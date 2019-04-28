use super::PyObject;
use std::os::raw::{c_char, c_int};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyMarshal_WriteObjectToString")]
    pub fn PyMarshal_WriteObjectToString(object: *mut PyObject, version: c_int) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyMarshal_ReadObjectFromString")]
    pub fn PyMarshal_ReadObjectFromString(data: *const c_char, len: isize) -> *mut PyObject;
}
