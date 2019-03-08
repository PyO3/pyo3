use ffi2::object::PyObject;
use ffi2::pyport::Py_ssize_t;
use std::os::raw::{c_char, c_int};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyErr_WarnEx")]
    pub fn PyErr_WarnEx(
        category: *mut PyObject,
        msg: *const c_char,
        stacklevel: Py_ssize_t,
    ) -> c_int;
    pub fn PyErr_WarnExplicit(
        arg1: *mut PyObject,
        arg2: *const c_char,
        arg3: *const c_char,
        arg4: c_int,
        arg5: *const c_char,
        arg6: *mut PyObject,
    ) -> c_int;
}

#[inline]
#[cfg_attr(PyPy, link_name = "PyPyErr_Warn")]
pub unsafe fn PyErr_Warn(category: *mut PyObject, msg: *const c_char) -> c_int {
    PyErr_WarnEx(category, msg, 1)
}
