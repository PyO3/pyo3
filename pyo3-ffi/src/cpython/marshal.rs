use crate::{PyObject, Py_ssize_t};
use libc::FILE;
use std::ffi::{c_char, c_int, c_long};

#[cfg(Py_3_15)]
pub const Py_MARSHAL_VERSION: c_int = 6;

#[cfg(not(Py_3_15))]
pub const Py_MARSHAL_VERSION: c_int = 5;

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyMarshal_WriteObjectToString")]
    #[cfg(not(Py_LIMITED_API))]
    pub fn PyMarshal_WriteObjectToString(object: *mut PyObject, version: c_int) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyMarshal_ReadObjectFromString")]
    pub fn PyMarshal_ReadObjectFromString(data: *const c_char, len: Py_ssize_t) -> *mut PyObject;

    pub fn PyMarshal_WriteLongToFile(value: c_long, file: *mut FILE, version: c_int);

    pub fn PyMarshal_WriteObjectToFile(object: *mut PyObject, file: *mut FILE, version: c_int);

    pub fn PyMarshal_ReadLongFromFile(file: *mut FILE) -> c_long;

    pub fn PyMarshal_ReadShortFromFile(file: *mut FILE) -> c_int;

    pub fn PyMarshal_ReadObjectFromFile(file: *mut FILE) -> *mut PyObject;

    pub fn PyMarshal_ReadLastObjectFromFile(file: *mut FILE) -> *mut PyObject;
}
