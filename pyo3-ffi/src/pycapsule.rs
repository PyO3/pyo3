use crate::object::*;
use std::ffi::{c_char, c_int, c_void};
use std::ptr::addr_of_mut;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_Type")]
    pub static mut PyCapsule_Type: PyTypeObject;
}

pub type PyCapsule_Destructor = unsafe extern "C" fn(o: *mut PyObject);

#[inline]
pub unsafe fn PyCapsule_CheckExact(ob: *mut PyObject) -> c_int {
    (Py_TYPE(ob) == addr_of_mut!(PyCapsule_Type)) as c_int
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_New")]
    pub fn PyCapsule_New(
        pointer: *mut c_void,
        name: *const c_char,
        destructor: Option<PyCapsule_Destructor>,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_GetPointer")]
    pub fn PyCapsule_GetPointer(capsule: *mut PyObject, name: *const c_char) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_GetDestructor")]
    pub fn PyCapsule_GetDestructor(capsule: *mut PyObject) -> Option<PyCapsule_Destructor>;
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_GetName")]
    pub fn PyCapsule_GetName(capsule: *mut PyObject) -> *const c_char;
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_GetContext")]
    pub fn PyCapsule_GetContext(capsule: *mut PyObject) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_IsValid")]
    pub fn PyCapsule_IsValid(capsule: *mut PyObject, name: *const c_char) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_SetPointer")]
    pub fn PyCapsule_SetPointer(capsule: *mut PyObject, pointer: *mut c_void) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_SetDestructor")]
    pub fn PyCapsule_SetDestructor(
        capsule: *mut PyObject,
        destructor: Option<PyCapsule_Destructor>,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_SetName")]
    pub fn PyCapsule_SetName(capsule: *mut PyObject, name: *const c_char) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_SetContext")]
    pub fn PyCapsule_SetContext(capsule: *mut PyObject, context: *mut c_void) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyCapsule_Import")]
    pub fn PyCapsule_Import(name: *const c_char, no_block: c_int) -> *mut c_void;
}
