use std::os::raw::c_int;
use ffi2::pyport::Py_ssize_t;
use ffi2::object::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyClassObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub cl_bases: *mut PyObject,
    pub cl_dict: *mut PyObject,
    pub cl_name: *mut PyObject,
    pub cl_getattr: *mut PyObject,
    pub cl_setattr: *mut PyObject,
    pub cl_delattr: *mut PyObject,
    pub cl_weakreflist: *mut PyObject,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyInstanceObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub in_class: *mut PyClassObject,
    pub in_dict: *mut PyObject,
    pub in_weakreflist: *mut PyObject,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyMethodObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub im_func: *mut PyObject,
    pub im_self: *mut PyObject,
    pub im_class: *mut PyObject,
    pub im_weakreflist: *mut PyObject,
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyClass_Type: PyTypeObject;
    pub static mut PyInstance_Type: PyTypeObject;
    pub static mut PyMethod_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyClass_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyClass_Type;
    (Py_TYPE(op) == u) as c_int
}

#[inline(always)]
pub unsafe fn PyInstance_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyInstance_Type;
    (Py_TYPE(op) == u) as c_int
}

#[inline(always)]
pub unsafe fn PyMethod_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyMethod_Type;
    (Py_TYPE(op) == u) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyClass_New(arg1: *mut PyObject, arg2: *mut PyObject,
                       arg3: *mut PyObject) -> *mut PyObject;
    pub fn PyInstance_New(arg1: *mut PyObject, arg2: *mut PyObject,
                          arg3: *mut PyObject) -> *mut PyObject;
    pub fn PyInstance_NewRaw(arg1: *mut PyObject, arg2: *mut PyObject)
                             -> *mut PyObject;
    pub fn PyMethod_New(arg1: *mut PyObject, arg2: *mut PyObject,
                        arg3: *mut PyObject) -> *mut PyObject;
    pub fn PyMethod_Function(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyMethod_Self(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyMethod_Class(arg1: *mut PyObject) -> *mut PyObject;
    fn _PyInstance_Lookup(pinst: *mut PyObject, name: *mut PyObject)
                          -> *mut PyObject;
    pub fn PyClass_IsSubclass(arg1: *mut PyObject, arg2: *mut PyObject)
                              -> c_int;
    pub fn PyMethod_ClearFreeList() -> c_int;
}

#[inline(always)]
pub unsafe fn PyMethod_GET_FUNCTION(meth : *mut PyObject) -> *mut PyObject {
    (*(meth as *mut PyMethodObject)).im_func
}

#[inline(always)]
pub unsafe fn PyMethod_GET_SELF(meth : *mut PyObject) -> *mut PyObject {
    (*(meth as *mut PyMethodObject)).im_self
}

#[inline(always)]
pub unsafe fn PyMethod_GET_CLASS(meth : *mut PyObject) -> *mut PyObject {
    (*(meth as *mut PyMethodObject)).im_class
}

