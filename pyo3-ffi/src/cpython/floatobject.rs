#[cfg(GraalPy)]
use crate::PyFloat_AsDouble;
#[cfg(not(GraalPy))]
use crate::PyFloat_Check;
use crate::PyObject;
#[cfg(Py_3_11)]
use core::ffi::{c_char, c_int};
use core::ffi::c_double;

#[repr(C)]
pub struct PyFloatObject {
    pub ob_base: PyObject,
    pub ob_fval: c_double,
}

#[inline]
#[cfg(not(GraalPy))]
unsafe fn _PyFloat_CAST(op: *mut PyObject) -> *mut PyFloatObject {
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

extern_libpython! {
    #[cfg(Py_3_11)]
    pub fn PyFloat_Pack2(x: c_double, p: *mut c_char, le: c_int) -> c_int;
    #[cfg(Py_3_11)]
    pub fn PyFloat_Pack4(x: c_double, p: *mut c_char, le: c_int) -> c_int;
    #[cfg(Py_3_11)]
    pub fn PyFloat_Pack8(x: c_double, p: *mut c_char, le: c_int) -> c_int;

    #[cfg(Py_3_11)]
    pub fn PyFloat_Unpack2(p: *const c_char, le: c_int) -> c_double;
    #[cfg(Py_3_11)]
    pub fn PyFloat_Unpack4(p: *const c_char, le: c_int) -> c_double;
    #[cfg(Py_3_11)]
    pub fn PyFloat_Unpack8(p: *const c_char, le: c_int) -> c_double;
}
