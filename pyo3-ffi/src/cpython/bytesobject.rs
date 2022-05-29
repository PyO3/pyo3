use crate::object::*;
use crate::pyport::{Py_hash_t, Py_ssize_t};
use std::os::raw::{c_char, c_int};

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyBytesObject {
    pub ob_base: PyVarObject,
    pub ob_shash: Py_hash_t,
    pub ob_sval: [c_char; 1],
}

#[cfg(Py_LIMITED_API)]
opaque_struct!(PyBytesObject);

extern "C" {
    pub fn _PyBytes_Resize(bytes: *mut *mut PyObject, newsize: Py_ssize_t) -> c_int;
}
