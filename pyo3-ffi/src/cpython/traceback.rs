use crate::object::PyObject;
use crate::PyFrameObject;
use core::ffi::c_int;

#[repr(C)]
pub struct PyTracebackObject {
    ob_base: PyObject,
    tb_next: *mut PyTracebackObject,
    tb_frame: *mut PyFrameObject,
    tb_lasti: c_int,
    tb_lineno: c_int,
}
