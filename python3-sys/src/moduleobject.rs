use libc::{c_char, c_int, c_void};
use pyport::Py_ssize_t;
use object::*;
use methodobject::PyMethodDef;

extern "C" {
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

extern "C" {
    pub fn PyModule_NewObject(name: *mut PyObject) -> *mut PyObject;
    pub fn PyModule_New(name: *const c_char) -> *mut PyObject;
    pub fn PyModule_GetDict(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyModule_GetNameObject(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyModule_GetName(arg1: *mut PyObject) -> *const c_char;
    pub fn PyModule_GetFilename(arg1: *mut PyObject) -> *const c_char;
    pub fn PyModule_GetFilenameObject(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyModule_GetDef(arg1: *mut PyObject) -> *mut PyModuleDef;
    pub fn PyModule_GetState(arg1: *mut PyObject) -> *mut c_void;
}

#[repr(C)]
#[derive(Copy)]
pub struct PyModuleDef_Base {
    pub ob_base: PyObject,
    pub m_init: Option<extern "C" fn() -> *mut PyObject>,
    pub m_index: Py_ssize_t,
    pub m_copy: *mut PyObject,
}
impl ::std::clone::Clone for PyModuleDef_Base {
    fn clone(&self) -> PyModuleDef_Base { *self }
}

pub const PyModuleDef_HEAD_INIT: PyModuleDef_Base = PyModuleDef_Base {
    ob_base: PyObject_HEAD_INIT,
    m_init: None,
    m_index: 0,
    m_copy: 0 as *mut PyObject
};

#[repr(C)]
#[derive(Copy)]
pub struct PyModuleDef {
    pub m_base: PyModuleDef_Base,
    pub m_name: *const c_char,
    pub m_doc: *const c_char,
    pub m_size: Py_ssize_t,
    pub m_methods: *mut PyMethodDef,
    pub m_reload: Option<inquiry>,
    pub m_traverse: Option<traverseproc>,
    pub m_clear: Option<inquiry>,
    pub m_free: Option<freefunc>,
}
impl ::std::clone::Clone for PyModuleDef {
    fn clone(&self) -> PyModuleDef { *self }
}


