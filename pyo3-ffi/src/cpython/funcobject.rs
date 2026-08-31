use crate::{PyObject, Py_IS_TYPE};
use core::ffi::c_int;

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

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyFunction_Type")]
    pub static mut PyFunction_Type: crate::PyTypeObject;
}

#[inline]
pub unsafe fn PyFunction_Check(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyFunction_Type)
}

extern_libpython! {
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

#[inline]
#[cfg(all(not(PyPy), not(GraalPy)))]
pub unsafe fn _PyFunction_CAST(func: *mut PyObject) -> *mut PyFunctionObject {
    assert_eq!(PyFunction_Check(func), 1);
    func.cast::<PyFunctionObject>()
}

#[inline]
#[cfg(all(not(PyPy), not(GraalPy)))]
pub unsafe fn PyFunction_GET_CODE(func: *mut PyObject) -> *mut PyObject {
    (*_PyFunction_CAST(func)).func_code
}

#[inline]
#[cfg(all(not(PyPy), not(GraalPy)))]
pub unsafe fn PyFunction_GET_GLOBALS(func: *mut PyObject) -> *mut PyObject {
    (*_PyFunction_CAST(func)).func_globals
}

#[inline]
#[cfg(all(not(PyPy), not(GraalPy)))]
pub unsafe fn PyFunction_GET_MODULE(func: *mut PyObject) -> *mut PyObject {
    (*_PyFunction_CAST(func)).func_module
}

#[inline]
#[cfg(all(not(PyPy), not(GraalPy)))]
pub unsafe fn PyFunction_GET_DEFAULTS(func: *mut PyObject) -> *mut PyObject {
    (*_PyFunction_CAST(func)).func_defaults
}

#[inline]
#[cfg(all(not(PyPy), not(GraalPy)))]
pub unsafe fn PyFunction_GET_KW_DEFAULTS(func: *mut PyObject) -> *mut PyObject {
    (*_PyFunction_CAST(func)).func_kwdefaults
}

#[inline]
#[cfg(all(not(PyPy), not(GraalPy)))]
pub unsafe fn PyFunction_GET_CLOSURE(func: *mut PyObject) -> *mut PyObject {
    (*_PyFunction_CAST(func)).func_closure
}

#[inline]
#[cfg(all(not(PyPy), not(GraalPy)))]
pub unsafe fn PyFunction_GET_ANNOTATIONS(func: *mut PyObject) -> *mut PyObject {
    (*_PyFunction_CAST(func)).func_annotations
}

extern_libpython! {
    pub static mut PyClassMethod_Type: crate::PyTypeObject;
    pub static mut PyStaticMethod_Type: crate::PyTypeObject;

    pub fn PyClassMethod_New(ob: *mut PyObject) -> *mut PyObject;
    pub fn PyStaticMethod_New(ob: *mut PyObject) -> *mut PyObject;
}
