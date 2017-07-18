use std::os::raw::{c_char, c_int, c_void};
use ffi2::pyport::Py_ssize_t;
use ffi2::object::*;
use ffi2::methodobject::PyMethodDef;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyModule_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyModule_Check(op : *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyModule_Type)
}

#[inline(always)]
pub unsafe fn PyModule_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyModule_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyModule_NewObject(name: *mut PyObject) -> *mut PyObject;
    pub fn PyModule_New(name: *const c_char) -> *mut PyObject;
    pub fn PyModule_GetDict(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyModule_GetNameObject(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyModule_GetName(arg1: *mut PyObject) -> *const c_char;
    pub fn PyModule_GetFilename(arg1: *mut PyObject) -> *const c_char;
    pub fn PyModule_GetFilenameObject(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyModule_GetDef(arg1: *mut PyObject) -> *mut PyModuleDef;
    pub fn PyModule_GetState(arg1: *mut PyObject) -> *mut c_void;
    pub fn PyModuleDef_Init(arg1: *mut PyModuleDef) -> *mut PyObject;
    pub static mut PyModuleDef_Type: PyTypeObject;
}

#[repr(C)]
#[derive(Copy)]
pub struct PyModuleDef_Base {
    pub ob_base: PyObject,
    pub m_init: Option<extern "C" fn() -> *mut PyObject>,
    pub m_index: Py_ssize_t,
    pub m_copy: *mut PyObject,
}
impl Clone for PyModuleDef_Base {
    fn clone(&self) -> PyModuleDef_Base { *self }
}

pub const PyModuleDef_HEAD_INIT: PyModuleDef_Base = PyModuleDef_Base {
    ob_base: PyObject_HEAD_INIT,
    m_init: None,
    m_index: 0,
    m_copy: ::std::ptr::null_mut()
};

#[repr(C)]
#[derive(Copy)]
pub struct PyModuleDef_Slot {
    pub slot: c_int,
    pub value: *mut c_void,
}
impl Clone for PyModuleDef_Slot {
    fn clone(&self) -> PyModuleDef_Slot { *self }
}

pub const Py_mod_create : c_int = 1;
pub const Py_mod_exec : c_int = 2;

#[repr(C)]
#[derive(Copy)]
pub struct PyModuleDef {
    pub m_base: PyModuleDef_Base,
    pub m_name: *const c_char,
    pub m_doc: *const c_char,
    pub m_size: Py_ssize_t,
    pub m_methods: *mut PyMethodDef,
    pub m_slots: *mut PyModuleDef_Slot,
    pub m_traverse: Option<traverseproc>,
    pub m_clear: Option<inquiry>,
    pub m_free: Option<freefunc>,
}
impl Clone for PyModuleDef {
    fn clone(&self) -> PyModuleDef { *self }
}

pub const PyModuleDef_INIT: PyModuleDef = PyModuleDef {
    m_base: PyModuleDef_HEAD_INIT,
    m_name: ::std::ptr::null(),
    m_doc: ::std::ptr::null(),
    m_size: 0,
    m_methods: ::std::ptr::null_mut(),
    m_slots: ::std::ptr::null_mut(),
    m_traverse: None,
    m_clear: None,
    m_free: None
};
