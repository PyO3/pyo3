use std::os::raw::c_int;

use crate::object::{PyObject, PyTypeObject, Py_TYPE};

// skipped PyFunctionObject

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyFunction_Type")]
    pub static mut PyFunction_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyFunction_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyFunction_Type) as c_int
}

extern "C" {
    pub fn PyFunction_New(code: *mut PyObject, globals: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_NewWithQualName(
        code: *mut PyObject,
        globals: *mut PyObject,
        qualname: *mut PyObject,
    ) -> *mut PyObject;
    pub fn PyFunction_GetCode(op: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_GetGlobals(op: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_GetModule(op: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_GetDefaults(op: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_SetDefaults(op: *mut PyObject, defaults: *mut PyObject) -> c_int;
    pub fn PyFunction_GetKwDefaults(op: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_SetKwDefaults(op: *mut PyObject, defaults: *mut PyObject) -> c_int;
    pub fn PyFunction_GetClosure(op: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_SetClosure(op: *mut PyObject, closure: *mut PyObject) -> c_int;
    pub fn PyFunction_GetAnnotations(op: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_SetAnnotations(op: *mut PyObject, annotations: *mut PyObject) -> c_int;
}

// skipped _PyFunction_Vectorcall
// skipped PyFunction_GET_CODE
// skipped PyFunction_GET_GLOBALS
// skipped PyFunction_GET_MODULE
// skipped PyFunction_GET_DEFAULTS
// skipped PyFunction_GET_KW_DEFAULTS
// skipped PyFunction_GET_CLOSURE
// skipped PyFunction_GET_ANNOTATIONS

// skipped PyClassMethod_Type
// skipped PyStaticMethod_Type
// skipped PyClassMethod_New
// skipped PyStaticMethod_New
