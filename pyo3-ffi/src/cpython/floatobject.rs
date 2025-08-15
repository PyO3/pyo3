#[cfg(GraalPy)]
use crate::PyFloat_AsDouble;
use crate::{PyFloat_Check, PyObject};
use std::ffi::c_double;

#[repr(C)]
pub struct PyFloatObject {
    pub ob_base: PyObject,
    pub ob_fval: c_double,
}

#[inline]
pub unsafe fn _PyFloat_CAST(op: *mut PyObject) -> *mut PyFloatObject {
    debug_assert_eq!(PyFloat_Check(op), 1);
    op.cast()
}

#[inline]
pub unsafe fn PyFloat_AS_DOUBLE(op: *mut PyObject) -> c_double {
    #[cfg(not(GraalPy))]
    return (*_PyFloat_CAST(op)).ob_fval;
    #[cfg(GraalPy)]
    return PyFloat_AsDouble(op);
}

// skipped PyFloat_Pack2
// skipped PyFloat_Pack4
// skipped PyFloat_Pack8

// skipped PyFloat_Unpack2
// skipped PyFloat_Unpack4
// skipped PyFloat_Unpack8
