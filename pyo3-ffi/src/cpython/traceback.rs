use crate::object::PyObject;
use crate::PyFrameObject;
use core::ffi::c_int;

#[repr(C)]
pub struct PyTracebackObject {
    pub ob_base: PyObject,
    pub tb_next: *mut PyTracebackObject,
    pub tb_frame: *mut PyFrameObject,
    pub tb_lasti: c_int,
    pub tb_lineno: c_int,
}
