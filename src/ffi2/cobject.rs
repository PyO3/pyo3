use crate::ffi2::object::*;
use std::os::raw::{c_char, c_int, c_void};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyCObject_Type")]
    pub static mut PyCObject_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyCObject_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyCObject_Type) as c_int
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyCObject_FromVoidPtr")]
    pub fn PyCObject_FromVoidPtr(
        cobj: *mut c_void,
        destruct: Option<unsafe extern "C" fn(arg1: *mut c_void)>,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyCObject_FromVoidPtrAndDesc")]
    pub fn PyCObject_FromVoidPtrAndDesc(
        cobj: *mut c_void,
        desc: *mut c_void,
        destruct: Option<unsafe extern "C" fn(arg1: *mut c_void, arg2: *mut c_void)>,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyCObject_AsVoidPtr")]
    pub fn PyCObject_AsVoidPtr(arg1: *mut PyObject) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyCObject_GetDesc")]
    pub fn PyCObject_GetDesc(arg1: *mut PyObject) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyCObject_Import")]
    pub fn PyCObject_Import(module_name: *mut c_char, cobject_name: *mut c_char) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyCObject_SetVoidPtr")]
    pub fn PyCObject_SetVoidPtr(_self: *mut PyObject, cobj: *mut c_void) -> c_int;
}
