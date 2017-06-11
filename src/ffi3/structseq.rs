use std::os::raw::{c_char, c_int};
use ffi3::pyport::Py_ssize_t;
use ffi3::object::{PyObject, PyTypeObject};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyStructSequence_Field {
    pub name: *mut c_char,
    pub doc: *mut c_char,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyStructSequence_Desc {
    pub name: *mut c_char,
    pub doc: *mut c_char,
    pub fields: *mut PyStructSequence_Field,
    pub n_in_sequence: c_int,
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyStructSequence_NewType(desc: *mut PyStructSequence_Desc)
     -> *mut PyTypeObject;
    pub fn PyStructSequence_New(_type: *mut PyTypeObject) -> *mut PyObject;
    pub fn PyStructSequence_SetItem(arg1: *mut PyObject, arg2: Py_ssize_t,
                                    arg3: *mut PyObject) -> ();
    pub fn PyStructSequence_GetItem(arg1: *mut PyObject, arg2: Py_ssize_t)
     -> *mut PyObject;
}

