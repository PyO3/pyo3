use crate::object::*;
#[cfg(not(Py_LIMITED_API))]
use std::ffi::c_char;

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

// skipped private _PyBytes_Resize

#[cfg(not(Py_LIMITED_API))]
#[inline]
pub unsafe fn PyBytes_AS_STRING(op: *mut PyObject) -> *const c_char {
    #[cfg(not(any(PyPy, GraalPy)))]
    return &(*op.cast::<PyBytesObject>()).ob_sval as *const c_char;
    #[cfg(any(PyPy, GraalPy))]
    return crate::PyBytes_AsString(op);
}
