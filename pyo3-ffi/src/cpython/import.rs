use crate::{PyInterpreterState, PyObject};
#[cfg(not(PyPy))]
use std::ffi::c_uchar;
use std::ffi::{c_char, c_int};

// skipped PyInit__imp

extern "C" {
    // skipped private _PyImport_IsInitialized
    // skipped _PyImport_GetModuleId
    // skipped private _PyImport_SetModule
    // skipped private _PyImport_SetModuleString
    // skipped private _PyImport_AcquireLock
    // skipped private _PyImport_ReleaseLock
    // skipped private _PyImport_FixupBuiltin
    // skipped private _PyImport_FixupExtensionObject
}

#[cfg(not(PyPy))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _inittab {
    pub name: *const c_char,
    pub initfunc: Option<unsafe extern "C" fn() -> *mut PyObject>,
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg(not(PyPy))]
    pub static mut PyImport_Inittab: *mut _inittab;
}

extern "C" {
    #[cfg(not(PyPy))]
    pub fn PyImport_ExtendInittab(newtab: *mut _inittab) -> c_int;
}

#[cfg(not(PyPy))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _frozen {
    pub name: *const c_char,
    pub code: *const c_uchar,
    pub size: c_int,
    #[cfg(Py_3_11)]
    pub is_package: c_int,
    #[cfg(all(Py_3_11, not(Py_3_13)))]
    pub get_code: Option<unsafe extern "C" fn() -> *mut PyObject>,
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg(not(PyPy))]
    pub static mut PyImport_FrozenModules: *const _frozen;
}

// skipped _PyImport_FrozenBootstrap
// skipped _PyImport_FrozenStdlib
// skipped _PyImport_FrozenTest
