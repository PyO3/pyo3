use std::os::raw::c_int;
use ffi2::object::*;
use ffi2::pyport::Py_ssize_t;
use ffi2::frameobject::PyFrameObject;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyTracebackObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub tb_next: *mut PyTracebackObject,
    pub tb_frame: *mut PyFrameObject,
    pub tb_lasti: c_int,
    pub tb_lineno: c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyTraceBack_Here(arg1: *mut PyFrameObject) -> c_int;
    pub fn PyTraceBack_Print(arg1: *mut PyObject, arg2: *mut PyObject)
     -> c_int;
     
    pub static mut PyTraceBack_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyTraceBack_Check(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyTraceBack_Type) as c_int
}

