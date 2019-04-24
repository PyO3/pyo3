use super::PyObject;
use libc::FILE;
use std::os::raw::{c_char, c_int, c_long};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyMarshal_WriteLongToFile")]
    pub fn PyMarshal_WriteLongToFile(value: c_long, file: *mut FILE, version: c_int);

    #[cfg_attr(PyPy, link_name = "PyMarshal_WriteObjectToFile")]
    pub fn PyMarshal_WriteObjectToFile(value: *mut PyObject, file: *mut FILE, version: c_int);

    #[cfg_attr(PyPy, link_name = "PyMarshal_WriteObjectToString")]
    pub fn PyMarshal_WriteObjectToString(object: *mut PyObject, version: c_int) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyMarshal_ReadLongFromFile")]
    pub fn PyMarshal_ReadLongFromFile(file: *mut FILE) -> c_long;

    #[cfg_attr(PyPy, link_name = "PyMarshal_ReadShortFromFile")]
    pub fn PyMarshal_ReadShortFromFile(file: *mut FILE) -> c_int;

    #[cfg_attr(PyPy, link_name = "PyMarshal_ReadObjectFromFile")]
    pub fn PyMarshal_ReadObjectFromFile(file: *mut FILE) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyMarshal_ReadLastObjectFromFile")]
    pub fn PyMarshal_ReadLastObjectFromFile(file: *mut FILE) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyMarshal_ReadObjectFromString")]
    pub fn PyMarshal_ReadObjectFromString(data: *const c_char, len: isize) -> *mut PyObject;
}
