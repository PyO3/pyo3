#[cfg(any(not(PyPy), Py_3_14))]
use crate::PyObject;
#[cfg(any(not(PyPy), Py_3_14))]
use std::ffi::c_char;
#[cfg(not(PyPy))]
use std::ffi::{c_int, c_uchar};

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

    #[cfg(Py_3_14)]
    pub fn PyImport_ImportModuleAttr(
        mod_name: *mut PyObject,
        attr_name: *mut PyObject,
    ) -> *mut PyObject;
    #[cfg(Py_3_14)]
    pub fn PyImport_ImportModuleAttrString(
        mod_name: *const c_char,
        attr_name: *const c_char,
    ) -> *mut PyObject;
}
