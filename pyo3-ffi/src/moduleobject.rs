use crate::methodobject::PyMethodDef;
use crate::object::*;
use crate::pyport::Py_ssize_t;
use std::ffi::{c_char, c_int, c_void};
use std::ptr::addr_of_mut;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyModule_Type")]
    pub static mut PyModule_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyModule_Check(op: *mut PyObject) -> c_int {
    PyObject_TypeCheck(op, addr_of_mut!(PyModule_Type))
}

#[inline]
pub unsafe fn PyModule_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyModule_Type)) as c_int
}

extern "C" {
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
pub struct PyModuleDef_Base {
    pub ob_base: PyObject,
    pub m_init: Option<extern "C" fn() -> *mut PyObject>,
    pub m_index: Py_ssize_t,
    pub m_copy: *mut PyObject,
}

#[allow(clippy::declare_interior_mutable_const)]
pub const PyModuleDef_HEAD_INIT: PyModuleDef_Base = PyModuleDef_Base {
    ob_base: PyObject_HEAD_INIT,
    m_init: None,
    m_index: 0,
    m_copy: std::ptr::null_mut(),
};

#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PyModuleDef_Slot {
    pub slot: c_int,
    pub value: *mut c_void,
}

impl Default for PyModuleDef_Slot {
    fn default() -> PyModuleDef_Slot {
        PyModuleDef_Slot {
            slot: 0,
            value: std::ptr::null_mut(),
        }
    }
}

pub const Py_mod_create: c_int = 1;
pub const Py_mod_exec: c_int = 2;
#[cfg(Py_3_12)]
pub const Py_mod_multiple_interpreters: c_int = 3;
#[cfg(Py_3_13)]
pub const Py_mod_gil: c_int = 4;

// skipped private _Py_mod_LAST_SLOT

#[cfg(Py_3_12)]
#[allow(clippy::zero_ptr)] // matches the way that the rest of these constants are defined
pub const Py_MOD_MULTIPLE_INTERPRETERS_NOT_SUPPORTED: *mut c_void = 0 as *mut c_void;
#[cfg(Py_3_12)]
pub const Py_MOD_MULTIPLE_INTERPRETERS_SUPPORTED: *mut c_void = 1 as *mut c_void;
#[cfg(Py_3_12)]
pub const Py_MOD_PER_INTERPRETER_GIL_SUPPORTED: *mut c_void = 2 as *mut c_void;

#[cfg(Py_3_13)]
#[allow(clippy::zero_ptr)] // matches the way that the rest of these constants are defined
pub const Py_MOD_GIL_USED: *mut c_void = 0 as *mut c_void;
#[cfg(Py_3_13)]
pub const Py_MOD_GIL_NOT_USED: *mut c_void = 1 as *mut c_void;

#[cfg(all(not(Py_LIMITED_API), Py_GIL_DISABLED))]
extern "C" {
    pub fn PyUnstable_Module_SetGIL(module: *mut PyObject, gil: *mut c_void) -> c_int;
}

#[repr(C)]
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
