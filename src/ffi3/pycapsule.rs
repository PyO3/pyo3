use std::os::raw::{c_void, c_char, c_int};
use ffi3::object::*;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyCapsule_Type: PyTypeObject;
}

pub type PyCapsule_Destructor = unsafe extern "C" fn(o: *mut PyObject);

#[inline]
pub unsafe fn PyCapsule_CheckExact(ob: *mut PyObject) -> c_int {
    (Py_TYPE(ob) == &mut PyCapsule_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyCapsule_New(pointer: *mut c_void,
                         name: *const c_char,
                         destructor: Option<PyCapsule_Destructor>) -> *mut PyObject;
    pub fn PyCapsule_GetPointer(capsule: *mut PyObject,
                                name: *const c_char)
     -> *mut c_void;
    pub fn PyCapsule_GetDestructor(capsule: *mut PyObject)
     -> Option<PyCapsule_Destructor>;
    pub fn PyCapsule_GetName(capsule: *mut PyObject) -> *const c_char;
    pub fn PyCapsule_GetContext(capsule: *mut PyObject)
     -> *mut c_void;
    pub fn PyCapsule_IsValid(capsule: *mut PyObject,
                             name: *const c_char) -> c_int;
    pub fn PyCapsule_SetPointer(capsule: *mut PyObject,
                                pointer: *mut c_void)
     -> c_int;
    pub fn PyCapsule_SetDestructor(capsule: *mut PyObject,
                                   destructor: Option<PyCapsule_Destructor>)
     -> c_int;
    pub fn PyCapsule_SetName(capsule: *mut PyObject,
                             name: *const c_char) -> c_int;
    pub fn PyCapsule_SetContext(capsule: *mut PyObject,
                                context: *mut c_void)
     -> c_int;
    pub fn PyCapsule_Import(name: *const c_char,
                            no_block: c_int) -> *mut c_void;
}

