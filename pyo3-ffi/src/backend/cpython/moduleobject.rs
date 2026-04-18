use crate::moduleobject::PyModuleDef;
use crate::object::*;
use std::ffi::{c_char, c_void};

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyModule_Type")]
    pub static mut PyModule_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyModule_Check(op: *mut PyObject) -> std::ffi::c_int {
    PyObject_TypeCheck(op, &raw mut PyModule_Type)
}

#[inline]
pub unsafe fn PyModule_CheckExact(op: *mut PyObject) -> std::ffi::c_int {
    (Py_TYPE(op) == &raw mut PyModule_Type) as std::ffi::c_int
}

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyModule_NewObject")]
    pub fn PyModule_NewObject(name: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyModule_New")]
    pub fn PyModule_New(name: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyModule_GetDict")]
    pub fn PyModule_GetDict(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg(not(PyPy))]
    pub fn PyModule_GetNameObject(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyModule_GetName")]
    pub fn PyModule_GetName(arg1: *mut PyObject) -> *const c_char;
    #[cfg(not(all(windows, PyPy)))]
    #[deprecated(note = "Python 3.2")]
    pub fn PyModule_GetFilename(arg1: *mut PyObject) -> *const c_char;
    #[cfg(not(PyPy))]
    pub fn PyModule_GetFilenameObject(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyModule_GetDef")]
    pub fn PyModule_GetDef(arg1: *mut PyObject) -> *mut PyModuleDef;
    #[cfg_attr(PyPy, link_name = "PyPyModule_GetState")]
    pub fn PyModule_GetState(arg1: *mut PyObject) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyModuleDef_Init")]
    pub fn PyModuleDef_Init(arg1: *mut PyModuleDef) -> *mut PyObject;
}

extern_libpython! {
    pub static mut PyModuleDef_Type: PyTypeObject;
}

#[cfg(all(not(Py_LIMITED_API), Py_GIL_DISABLED))]
extern_libpython! {
    pub fn PyUnstable_Module_SetGIL(module: *mut PyObject, gil: *mut c_void) -> std::ffi::c_int;
}

#[cfg(Py_3_15)]
extern_libpython! {
    pub fn PyModule_FromSlotsAndSpec(
        slots: *const crate::moduleobject::PyModuleDef_Slot,
        spec: *mut PyObject,
    ) -> *mut PyObject;
    pub fn PyModule_Exec(_mod: *mut PyObject) -> std::ffi::c_int;
    pub fn PyModule_GetStateSize(_mod: *mut PyObject, result: *mut crate::pyport::Py_ssize_t)
        -> std::ffi::c_int;
    pub fn PyModule_GetToken(
        module: *mut PyObject,
        result: *mut *mut c_void,
    ) -> std::ffi::c_int;
}
