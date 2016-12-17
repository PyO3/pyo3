use libc::{c_char, c_int};
use pyport::Py_ssize_t;
use object::{PyObject, PyTypeObject};

#[repr(C)]
#[derive(Copy)]
pub struct PyStructSequence_Field {
    pub name: *mut c_char,
    pub doc: *mut c_char,
}
impl Clone for PyStructSequence_Field {
    #[inline] fn clone(&self) -> PyStructSequence_Field { *self }
}

#[repr(C)]
#[derive(Copy)]
pub struct PyStructSequence_Desc {
    pub name: *mut c_char,
    pub doc: *mut c_char,
    pub fields: *mut PyStructSequence_Field,
    pub n_in_sequence: c_int,
}
impl Clone for PyStructSequence_Desc {
    #[inline] fn clone(&self) -> PyStructSequence_Desc { *self }
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

