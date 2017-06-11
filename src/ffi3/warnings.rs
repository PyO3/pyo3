use std::os::raw::{c_char, c_int};
use ffi3::pyport::Py_ssize_t;
use ffi3::object::PyObject;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyErr_WarnEx(category: *mut PyObject,
                        message: *const c_char,
                        stack_level: Py_ssize_t) -> c_int;
    pub fn PyErr_WarnFormat(category: *mut PyObject, stack_level: Py_ssize_t,
                            format: *const c_char, ...)
     -> c_int;
    #[cfg(Py_3_6)]
    pub fn PyErr_ResourceWarning(
        source: *mut PyObject, stack_level: Py_ssize_t,
        format: *const c_char, ...) -> c_int;
    pub fn PyErr_WarnExplicit(category: *mut PyObject,
                              message: *const c_char,
                              filename: *const c_char,
                              lineno: c_int,
                              module: *const c_char,
                              registry: *mut PyObject) -> c_int;
}

