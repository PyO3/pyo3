use crate::object::{PyObject, PyTypeObject};
#[cfg(not(PyPy))]
use crate::pyport::Py_ssize_t;
use std::ffi::{c_char, c_int};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyStructSequence_Field {
    pub name: *const c_char,
    pub doc: *const c_char,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyStructSequence_Desc {
    pub name: *const c_char,
    pub doc: *const c_char,
    pub fields: *mut PyStructSequence_Field,
    pub n_in_sequence: c_int,
}

// skipped PyStructSequence_UnnamedField;

extern "C" {
    #[cfg(not(Py_LIMITED_API))]
    #[cfg_attr(PyPy, link_name = "PyPyStructSequence_InitType")]
    pub fn PyStructSequence_InitType(_type: *mut PyTypeObject, desc: *mut PyStructSequence_Desc);

    #[cfg(not(Py_LIMITED_API))]
    #[cfg_attr(PyPy, link_name = "PyPyStructSequence_InitType2")]
    pub fn PyStructSequence_InitType2(
        _type: *mut PyTypeObject,
        desc: *mut PyStructSequence_Desc,
    ) -> c_int;

    #[cfg(not(PyPy))]
    pub fn PyStructSequence_NewType(desc: *mut PyStructSequence_Desc) -> *mut PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyStructSequence_New")]
    pub fn PyStructSequence_New(_type: *mut PyTypeObject) -> *mut PyObject;
}

#[cfg(not(Py_LIMITED_API))]
pub type PyStructSequence = crate::PyTupleObject;

#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyStructSequence_SET_ITEM(op: *mut PyObject, i: Py_ssize_t, v: *mut PyObject) {
    crate::PyTuple_SET_ITEM(op, i, v)
}

#[cfg(not(any(Py_LIMITED_API, PyPy, GraalPy)))]
#[inline]
pub unsafe fn PyStructSequence_GET_ITEM(op: *mut PyObject, i: Py_ssize_t) -> *mut PyObject {
    crate::PyTuple_GET_ITEM(op, i)
}

extern "C" {
    #[cfg(not(PyPy))]
    pub fn PyStructSequence_SetItem(arg1: *mut PyObject, arg2: Py_ssize_t, arg3: *mut PyObject);

    #[cfg(not(PyPy))]
    pub fn PyStructSequence_GetItem(arg1: *mut PyObject, arg2: Py_ssize_t) -> *mut PyObject;
}
