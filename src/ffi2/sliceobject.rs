use std::os::raw::c_int;
use ffi2::pyport::Py_ssize_t;
use ffi2::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    static mut _Py_EllipsisObject: PyObject;
}

#[inline(always)]
pub unsafe fn Py_Ellipsis() -> *mut PyObject {
    &mut _Py_EllipsisObject
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PySliceObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub start: *mut PyObject,
    pub stop: *mut PyObject,
    pub step: *mut PyObject
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PySlice_Type: PyTypeObject;
    pub static mut PyEllipsis_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PySlice_Check(op: *mut PyObject) -> c_int {
     (Py_TYPE(op) == &mut PySlice_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PySlice_New(start: *mut PyObject, stop: *mut PyObject,
                       step: *mut PyObject) -> *mut PyObject;
    pub fn PySlice_GetIndices(r: *mut PyObject, length: Py_ssize_t,
                              start: *mut Py_ssize_t, stop: *mut Py_ssize_t,
                              step: *mut Py_ssize_t) -> c_int;
    pub fn PySlice_GetIndicesEx(r: *mut PyObject, length: Py_ssize_t,
                                start: *mut Py_ssize_t, stop: *mut Py_ssize_t,
                                step: *mut Py_ssize_t,
                                slicelength: *mut Py_ssize_t)
     -> c_int;
}

