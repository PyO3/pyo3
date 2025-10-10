use crate::object::PyObject;
use crate::pyport::Py_ssize_t;
use std::ffi::{c_char, c_int};

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyErr_WarnEx")]
    pub fn PyErr_WarnEx(
        category: *mut PyObject,
        message: *const c_char,
        stack_level: Py_ssize_t,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyErr_WarnFormat")]
    pub fn PyErr_WarnFormat(
        category: *mut PyObject,
        stack_level: Py_ssize_t,
        format: *const c_char,
        ...
    ) -> c_int;
    pub fn PyErr_ResourceWarning(
        source: *mut PyObject,
        stack_level: Py_ssize_t,
        format: *const c_char,
        ...
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyErr_WarnExplicit")]
    pub fn PyErr_WarnExplicit(
        category: *mut PyObject,
        message: *const c_char,
        filename: *const c_char,
        lineno: c_int,
        module: *const c_char,
        registry: *mut PyObject,
    ) -> c_int;
}
