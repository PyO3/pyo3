use ffi2::frameobject::PyFrameObject;
use ffi2::object::*;
use ffi2::pyport::Py_ssize_t;
use std::os::raw::c_int;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyTracebackObject {
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config = "Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub tb_next: *mut PyTracebackObject,
    pub tb_frame: *mut PyFrameObject,
    pub tb_lasti: c_int,
    pub tb_lineno: c_int,
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyTraceBack_Here")]
    pub fn PyTraceBack_Here(arg1: *mut PyFrameObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyTraceBack_Print")]
    pub fn PyTraceBack_Print(arg1: *mut PyObject, arg2: *mut PyObject) -> c_int;

    #[cfg_attr(PyPy, link_name = "PyPyTraceBack_Type")]
    pub static mut PyTraceBack_Type: PyTypeObject;
}

#[inline]
#[cfg_attr(PyPy, link_name = "PyPyTraceBack_Check")]
pub unsafe fn PyTraceBack_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyTraceBack_Type) as c_int
}
