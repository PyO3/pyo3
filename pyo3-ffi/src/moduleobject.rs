use crate::methodobject::PyMethodDef;
use crate::object::*;
use crate::pyport::Py_ssize_t;
use std::ffi::{c_char, c_int, c_void};

#[repr(C)]
pub struct PyModuleDef_Base {
    pub ob_base: PyObject,
    // Rust function pointers are non-null so an Option is needed here.
    pub m_init: Option<extern "C" fn() -> *mut PyObject>,
    pub m_index: Py_ssize_t,
    pub m_copy: *mut PyObject,
}

#[allow(
    clippy::declare_interior_mutable_const,
    reason = "contains atomic refcount on free-threaded builds"
)]
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
#[cfg(Py_3_15)]
pub const Py_mod_abi: c_int = 5;
#[cfg(Py_3_15)]
pub const Py_mod_name: c_int = 6;
#[cfg(Py_3_15)]
pub const Py_mod_doc: c_int = 7;
#[cfg(Py_3_15)]
pub const Py_mod_state_size: c_int = 8;
#[cfg(Py_3_15)]
pub const Py_mod_methods: c_int = 9;
#[cfg(Py_3_15)]
pub const Py_mod_state_traverse: c_int = 10;
#[cfg(Py_3_15)]
pub const Py_mod_state_clear: c_int = 11;
#[cfg(Py_3_15)]
pub const Py_mod_state_free: c_int = 12;
#[cfg(Py_3_15)]
pub const Py_mod_token: c_int = 13;

// skipped private _Py_mod_LAST_SLOT

#[cfg(Py_3_12)]
#[allow(
    clippy::zero_ptr,
    reason = "matches the way that the rest of these constants are defined"
)]
pub const Py_MOD_MULTIPLE_INTERPRETERS_NOT_SUPPORTED: *mut c_void = 0 as *mut c_void;
#[cfg(Py_3_12)]
pub const Py_MOD_MULTIPLE_INTERPRETERS_SUPPORTED: *mut c_void = 1 as *mut c_void;
#[cfg(Py_3_12)]
pub const Py_MOD_PER_INTERPRETER_GIL_SUPPORTED: *mut c_void = 2 as *mut c_void;

#[cfg(Py_3_13)]
#[allow(
    clippy::zero_ptr,
    reason = "matches the way that the rest of these constants are defined"
)]
pub const Py_MOD_GIL_USED: *mut c_void = 0 as *mut c_void;
#[cfg(Py_3_13)]
pub const Py_MOD_GIL_NOT_USED: *mut c_void = 1 as *mut c_void;

#[cfg(all(not(Py_LIMITED_API), Py_GIL_DISABLED))]
pub use crate::backend::current::moduleobject::PyUnstable_Module_SetGIL;

#[repr(C)]
pub struct PyModuleDef {
    pub m_base: PyModuleDef_Base,
    pub m_name: *const c_char,
    pub m_doc: *const c_char,
    pub m_size: Py_ssize_t,
    pub m_methods: *mut PyMethodDef,
    pub m_slots: *mut PyModuleDef_Slot,
    // Rust function pointers are non-null so an Option is needed here.
    pub m_traverse: Option<traverseproc>,
    pub m_clear: Option<inquiry>,
    pub m_free: Option<freefunc>,
}

// Runtime module APIs live in the backend dispatcher.
pub use crate::backend::current::moduleobject::{
    PyModule_Check, PyModule_CheckExact, PyModule_GetDef, PyModule_GetDict,
    PyModule_GetFilename, PyModule_GetFilenameObject, PyModule_GetName, PyModule_GetNameObject,
    PyModule_GetState, PyModule_New, PyModule_NewObject, PyModule_Type, PyModuleDef_Init,
    PyModuleDef_Type,
};

#[cfg(Py_3_15)]
pub use crate::backend::current::moduleobject::{
    PyModule_Exec, PyModule_FromSlotsAndSpec, PyModule_GetStateSize, PyModule_GetToken,
};
