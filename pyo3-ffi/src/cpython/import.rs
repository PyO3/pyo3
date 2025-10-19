use crate::{PyInterpreterState, PyObject};
#[cfg(not(PyPy))]
use std::ffi::c_uchar;
use std::ffi::{c_char, c_int};

// skipped PyInit__imp

extern "C" {
    pub fn _PyImport_IsInitialized(state: *mut PyInterpreterState) -> c_int;
    // skipped _PyImport_GetModuleId
    pub fn _PyImport_SetModule(name: *mut PyObject, module: *mut PyObject) -> c_int;
    pub fn _PyImport_SetModuleString(name: *const c_char, module: *mut PyObject) -> c_int;
    pub fn _PyImport_AcquireLock();
    pub fn _PyImport_ReleaseLock() -> c_int;
    #[cfg(not(Py_3_9))]
    pub fn _PyImport_FindBuiltin(name: *const c_char, modules: *mut PyObject) -> *mut PyObject;
    #[cfg(not(Py_3_11))]
    pub fn _PyImport_FindExtensionObject(a: *mut PyObject, b: *mut PyObject) -> *mut PyObject;
    pub fn _PyImport_FixupBuiltin(
        module: *mut PyObject,
        name: *const c_char,
        modules: *mut PyObject,
    ) -> c_int;
    pub fn _PyImport_FixupExtensionObject(
        a: *mut PyObject,
        b: *mut PyObject,
        c: *mut PyObject,
        d: *mut PyObject,
    ) -> c_int;
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
