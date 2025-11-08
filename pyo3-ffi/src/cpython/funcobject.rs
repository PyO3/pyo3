use std::ffi::c_int;
#[cfg(not(all(PyPy, not(Py_3_8))))]
use std::ptr::addr_of_mut;

use crate::PyObject;

#[cfg(all(not(any(PyPy, GraalPy)), not(Py_3_10)))]
#[repr(C)]
pub struct PyFunctionObject {
    pub ob_base: PyObject,
    pub func_code: *mut PyObject,
    pub func_globals: *mut PyObject,
    pub func_defaults: *mut PyObject,
    pub func_kwdefaults: *mut PyObject,
    pub func_closure: *mut PyObject,
    pub func_doc: *mut PyObject,
    pub func_name: *mut PyObject,
    pub func_dict: *mut PyObject,
    pub func_weakreflist: *mut PyObject,
    pub func_module: *mut PyObject,
    pub func_annotations: *mut PyObject,
    pub func_qualname: *mut PyObject,
    #[cfg(Py_3_8)]
    pub vectorcall: Option<crate::vectorcallfunc>,
}

#[cfg(all(not(any(PyPy, GraalPy)), Py_3_10))]
#[repr(C)]
pub struct PyFunctionObject {
    pub ob_base: PyObject,
    pub func_globals: *mut PyObject,
    pub func_builtins: *mut PyObject,
    pub func_name: *mut PyObject,
    pub func_qualname: *mut PyObject,
    pub func_code: *mut PyObject,
    pub func_defaults: *mut PyObject,
    pub func_kwdefaults: *mut PyObject,
    pub func_closure: *mut PyObject,
    pub func_doc: *mut PyObject,
    pub func_dict: *mut PyObject,
    pub func_weakreflist: *mut PyObject,
    pub func_module: *mut PyObject,
    pub func_annotations: *mut PyObject,
    #[cfg(Py_3_14)]
    pub func_annotate: *mut PyObject,
    #[cfg(Py_3_12)]
    pub func_typeparams: *mut PyObject,
    pub vectorcall: Option<crate::vectorcallfunc>,
    #[cfg(Py_3_11)]
    pub func_version: u32,
}

#[cfg(PyPy)]
#[repr(C)]
pub struct PyFunctionObject {
    pub ob_base: PyObject,
    pub func_name: *mut PyObject,
}

#[cfg(GraalPy)]
pub struct PyFunctionObject {
    pub ob_base: PyObject,
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg(not(all(PyPy, not(Py_3_8))))]
    #[cfg_attr(PyPy, link_name = "PyPyFunction_Type")]
    pub static mut PyFunction_Type: crate::PyTypeObject;
}

#[cfg(not(all(PyPy, not(Py_3_8))))]
#[inline]
pub unsafe fn PyFunction_Check(op: *mut PyObject) -> c_int {
    (crate::Py_TYPE(op) == addr_of_mut!(PyFunction_Type)) as c_int
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
