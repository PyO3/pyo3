use libc::{c_char, c_int};
use object::*;

#[link(name = "python2.7")]
extern "C" {
    pub static mut PyModule_Type: PyTypeObject;
    
    pub fn PyModule_New(name: *const c_char) -> *mut PyObject;
    pub fn PyModule_GetDict(module: *mut PyObject) -> *mut PyObject;
    pub fn PyModule_GetName(module: *mut PyObject) -> *mut c_char;
    pub fn PyModule_GetFilename(module: *mut PyObject) -> *mut c_char;
    //fn _PyModule_Clear(arg1: *mut PyObject);
}

#[inline(always)]
pub unsafe fn PyModule_Check(op : *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyModule_Type)
}

#[inline(always)]
pub unsafe fn PyModule_CheckExact(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyModule_Type;
    (Py_TYPE(op) == u) as c_int
}

