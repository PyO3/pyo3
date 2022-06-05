use crate::{PyInterpreterState, PyObject};
#[cfg(not(PyPy))]
use std::os::raw::c_uchar;
use std::os::raw::{c_char, c_int};

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
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg(not(PyPy))]
    pub static mut PyImport_FrozenModules: *const _frozen;
}
