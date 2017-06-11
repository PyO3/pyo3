use std::os::raw::c_int;
use ffi3::pyport::Py_ssize_t;
use ffi3::object::*;
use ffi3::frameobject::PyFrameObject;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyGenObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub gi_frame: *mut PyFrameObject,
    pub gi_running: c_int,
    pub gi_code: *mut PyObject,
    pub gi_weakreflist: *mut PyObject
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyGen_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyGen_Check(op: *mut PyObject) -> c_int {
     PyObject_TypeCheck(op, &mut PyGen_Type)
}

#[inline(always)]
pub unsafe fn PyGen_CheckExact(op: *mut PyObject) -> c_int {
     (Py_TYPE(op) == &mut PyGen_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyGen_New(frame: *mut PyFrameObject) -> *mut PyObject;
    pub fn PyGen_NeedsFinalizing(op: *mut PyGenObject) -> c_int;
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyCoro_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyCoro_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyCoro_Type)
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut _PyCoroWrapper_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyCoroWrapper_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut _PyCoroWrapper_Type)
}

#[cfg(Py_3_6)]
#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyAsyncGen_Type: PyTypeObject;
}

#[cfg(Py_3_6)]
#[inline(always)]
pub unsafe fn PyAsyncGen_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyAsyncGen_Type)
}

#[cfg(not(Py_3_6))]
#[inline(always)]
pub unsafe fn PyAsyncGen_Check(_op: *mut PyObject) -> c_int {
    0
}
