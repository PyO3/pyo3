use crate::methodobject::PyMethodDef;
use crate::object::*;
use crate::pyport::Py_ssize_t;
use std::os::raw::{c_char, c_int, c_void};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyModule_Type")]
    pub static mut PyModule_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyModule_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, &mut PyModule_Type)
}

#[inline]
pub unsafe fn PyModule_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyModule_Type) as c_int
}

extern "C" {
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
    pub fn PyModule_GetFilenameObject(arg1: *mut PyObject) -> *mut PyObject;
    // skipped non-limited _PyModule_Clear
    // skipped non-limited _PyModule_ClearDict
    // skipped non-limited _PyModuleSpec_IsInitializing
    #[cfg_attr(PyPy, link_name = "PyPyModule_GetDef")]
    pub fn PyModule_GetDef(arg1: *mut PyObject) -> *mut PyModuleDef;
    #[cfg_attr(PyPy, link_name = "PyPyModule_GetState")]
    pub fn PyModule_GetState(arg1: *mut PyObject) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyModuleDef_Init")]
    pub fn PyModuleDef_Init(arg1: *mut PyModuleDef) -> *mut PyObject;
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyModuleDef_Type: PyTypeObject;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyModuleDef_Base {
    pub ob_base: PyObject,
    pub m_init: Option<extern "C" fn() -> *mut PyObject>,
    pub m_index: Py_ssize_t,
    pub m_copy: *mut PyObject,
}

pub const PyModuleDef_HEAD_INIT: PyModuleDef_Base = PyModuleDef_Base {
    ob_base: PyObject_HEAD_INIT,
    m_init: None,
    m_index: 0,
    m_copy: std::ptr::null_mut(),
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyModuleDef_Slot {
    pub slot: c_int,
    pub value: *mut c_void,
}

pub const Py_mod_create: c_int = 1;
pub const Py_mod_exec: c_int = 2;

// skipped non-limited _Py_mod_LAST_SLOT

#[repr(C)]
#[derive(Copy, Clone)]
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
