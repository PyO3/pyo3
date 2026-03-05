use crate::object::*;
use crate::Py_ssize_t;
#[cfg(not(Py_LIMITED_API))]
use std::ffi::c_char;
use std::ffi::c_int;
#[cfg(Py_3_15)]
use std::ffi::c_void;

#[cfg(not(any(PyPy, GraalPy, Py_LIMITED_API)))]
#[repr(C)]
pub struct PyBytesObject {
    pub ob_base: PyVarObject,
    #[cfg_attr(
        Py_3_11,
        deprecated(note = "Deprecated in Python 3.11 and will be removed in a future version.")
    )]
    pub ob_shash: crate::Py_hash_t,
    pub ob_sval: [c_char; 1],
}

#[cfg(any(PyPy, GraalPy, Py_LIMITED_API))]
opaque_struct!(pub PyBytesObject);

extern "C" {
    #[cfg_attr(PyPy, link_name = "_PyPyBytes_Resize")]
    pub fn _PyBytes_Resize(bytes: *mut *mut PyObject, newsize: Py_ssize_t) -> c_int;
}

#[cfg(not(Py_LIMITED_API))]
#[inline]
pub unsafe fn PyBytes_AS_STRING(op: *mut PyObject) -> *const c_char {
    #[cfg(not(any(PyPy, GraalPy)))]
    return &(*op.cast::<PyBytesObject>()).ob_sval as *const c_char;
    #[cfg(any(PyPy, GraalPy))]
    return crate::PyBytes_AsString(op);
}

#[cfg(Py_3_15)]
opaque_struct!(pub PyBytesWriter);

#[cfg(Py_3_15)]
extern "C" {

    pub fn PyBytesWriter_Create(size: Py_ssize_t) -> *mut PyBytesWriter;

    pub fn PyBytesWriter_Discard(writer: *mut PyBytesWriter);

    pub fn PyBytesWriter_Finish(writer: *mut PyBytesWriter) -> *mut PyObject;

    pub fn PyBytesWriter_FinishWithSize(
        writer: *mut PyBytesWriter,
        size: Py_ssize_t,
    ) -> *mut PyObject;

    pub fn PyBytesWriter_GetData(writer: *mut PyBytesWriter) -> *mut c_void;

    pub fn PyBytesWriter_GetSize(writer: *mut PyBytesWriter) -> Py_ssize_t;

    pub fn PyBytesWriter_Resize(writer: *mut PyBytesWriter, size: Py_ssize_t) -> c_int;

    pub fn PyBytesWriter_Grow(writer: *mut PyBytesWriter, size: Py_ssize_t) -> c_int;
}
