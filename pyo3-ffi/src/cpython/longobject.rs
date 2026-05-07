use crate::{longobject::*, object::*};
use libc::size_t;
use std::ffi::{c_int, c_uchar};

// skipped _PyLong_CAST

extern_libpython! {
    // skipped PyUnstable_Long_IsCompact
    // skipped PyUnstable_Long_CompactValue

    #[cfg(Py_3_14)]
    pub fn PyLong_IsPositive(obj: *mut PyObject) -> c_int;
    #[cfg(Py_3_14)]
    pub fn PyLong_IsNegative(obj: *mut PyObject) -> c_int;
    #[cfg(Py_3_14)]
    pub fn PyLong_IsZero(obj: *mut PyObject) -> c_int;

    // skipped PyLong_GetSign

    // skipped _PyLong_Sign

    #[cfg(not(Py_LIMITED_API))]
    #[cfg_attr(PyPy, link_name = "_PyPyLong_NumBits")]
    #[cfg(not(Py_3_13))]
    #[doc(hidden)]
    pub fn _PyLong_NumBits(obj: *mut PyObject) -> size_t;

    #[cfg_attr(PyPy, link_name = "_PyPyLong_FromByteArray")]
    pub fn _PyLong_FromByteArray(
        bytes: *const c_uchar,
        n: size_t,
        little_endian: c_int,
        is_signed: c_int,
    ) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "_PyPyLong_AsByteArrayO")]
    pub fn _PyLong_AsByteArray(
        v: *mut PyLongObject,
        bytes: *mut c_uchar,
        n: size_t,
        little_endian: c_int,
        is_signed: c_int,
    ) -> c_int;

    // skipped _PyLong_GCD
}
