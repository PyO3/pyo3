use std::os::raw::{c_int, c_long};
use ffi2::pyport::Py_ssize_t;
use ffi2::object::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyWeakReference {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub wr_object: *mut PyObject,
    pub wr_callback: *mut PyObject,
    pub hash: c_long,
    pub wr_prev: *mut PyWeakReference,
    pub wr_next: *mut PyWeakReference
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    static mut _PyWeakref_RefType: PyTypeObject;
    static mut _PyWeakref_ProxyType: PyTypeObject;
    static mut _PyWeakref_CallableProxyType: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyWeakref_CheckRef(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut _PyWeakref_RefType)
}

#[inline(always)]
pub unsafe fn PyWeakref_CheckRefExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut _PyWeakref_RefType) as c_int
}

#[inline(always)]
pub unsafe fn PyWeakref_CheckProxy(op: *mut PyObject) -> c_int {
    ((Py_TYPE(op) == &mut _PyWeakref_ProxyType) ||
         (Py_TYPE(op) == &mut _PyWeakref_CallableProxyType)) as c_int
}

#[inline(always)]
pub unsafe fn PyWeakref_Check(op: *mut PyObject) -> c_int {
    (PyWeakref_CheckRef(op) != 0 || PyWeakref_CheckProxy(op) != 0) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyWeakref_NewRef(ob: *mut PyObject, callback: *mut PyObject)
     -> *mut PyObject;
    pub fn PyWeakref_NewProxy(ob: *mut PyObject, callback: *mut PyObject)
     -> *mut PyObject;
    pub fn PyWeakref_GetObject(_ref: *mut PyObject) -> *mut PyObject;
    
    pub fn _PyWeakref_GetWeakrefCount(head: *mut PyWeakReference) -> Py_ssize_t;
    pub fn _PyWeakref_ClearRef(slf: *mut PyWeakReference);
}

#[inline(always)]
pub unsafe fn PyWeakref_GET_OBJECT(_ref: *mut PyObject) -> *mut PyObject {
    let obj = (*(_ref as *mut PyWeakReference)).wr_object;
    if Py_REFCNT(obj) > 0 { obj } else { Py_None() }
}

