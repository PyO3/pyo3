use crate::object::*;
use crate::Py_ssize_t;
#[cfg(not(any(PyPy, GraalPy, Py_LIMITED_API)))]
use std::os::raw::c_char;
use std::os::raw::c_int;

#[cfg(not(any(PyPy, GraalPy, Py_LIMITED_API)))]
#[repr(C)]
pub struct PyBytesObject {
    pub ob_base: PyVarObject,
    pub ob_shash: crate::Py_hash_t,
    pub ob_sval: [c_char; 1],
}

#[cfg(any(PyPy, GraalPy, Py_LIMITED_API))]
opaque_struct!(PyBytesObject);

extern "C" {
    #[cfg_attr(PyPy, link_name = "_PyPyBytes_Resize")]
    pub fn _PyBytes_Resize(bytes: *mut *mut PyObject, newsize: Py_ssize_t) -> c_int;
}
