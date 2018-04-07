use std::os::raw::c_int;
use ffi2::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="PyPyFunction_Type")]
    pub static mut PyFunction_Type: PyTypeObject;
}

#[inline(always)]
#[cfg_attr(PyPy, link_name="PyPyFunction_Check")]
pub unsafe fn PyFunction_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyFunction_Type;
    (Py_TYPE(op) == u) as c_int
}


#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyFunction_New(code: *mut PyObject, globals: *mut PyObject)
     -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="PyPyFunction_GetCode")]
    pub fn PyFunction_GetCode(f: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_GetGlobals(f: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_GetModule(f: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_GetDefaults(f: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_SetDefaults(f: *mut PyObject, defaults: *mut PyObject)
     -> c_int;
    pub fn PyFunction_GetClosure(f: *mut PyObject) -> *mut PyObject;
    pub fn PyFunction_SetClosure(f: *mut PyObject, closure: *mut PyObject)
     -> c_int;
    
    pub static mut PyClassMethod_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name="PyPyStaticMethod_Type")]
    pub static mut PyStaticMethod_Type: PyTypeObject;
    
    #[cfg_attr(PyPy, link_name="PyPyClassMethod_New")]
    pub fn PyClassMethod_New(arg1: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name="PyPyStaticMethod_New")]
    pub fn PyStaticMethod_New(arg1: *mut PyObject) -> *mut PyObject;
}