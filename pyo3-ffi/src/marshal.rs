use super::{PyObject, Py_ssize_t};
use std::ffi::{c_char, c_int};

// skipped Py_MARSHAL_VERSION
// skipped PyMarshal_WriteLongToFile
// skipped PyMarshal_WriteObjectToFile

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyMarshal_WriteObjectToString")]
    pub fn PyMarshal_WriteObjectToString(object: *mut PyObject, version: c_int) -> *mut PyObject;

    // skipped non-limited PyMarshal_ReadLongFromFile
    // skipped non-limited PyMarshal_ReadShortFromFile
    // skipped non-limited PyMarshal_ReadObjectFromFile
    // skipped non-limited PyMarshal_ReadLastObjectFromFile

    #[cfg_attr(PyPy, link_name = "PyPyMarshal_ReadObjectFromString")]
    pub fn PyMarshal_ReadObjectFromString(data: *const c_char, len: Py_ssize_t) -> *mut PyObject;
}
