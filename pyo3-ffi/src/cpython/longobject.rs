use crate::{longobject::*, object::*};
use core::ffi::{c_int, c_uchar};
use libc::size_t;

// skipped _PyLong_CAST

extern_libpython! {
    #[cfg(Py_3_13)]
    pub fn PyLong_FromUnicodeObject(u: *mut PyObject, base: c_int) -> *mut PyObject;

    // skipped PyUnstable_Long_IsCompact
    // skipped PyUnstable_Long_CompactValue

    #[cfg(Py_3_14)]
    pub fn PyLong_IsPositive(obj: *mut PyObject) -> c_int;
    #[cfg(Py_3_14)]
    pub fn PyLong_IsNegative(obj: *mut PyObject) -> c_int;
    #[cfg(Py_3_14)]
    pub fn PyLong_IsZero(obj: *mut PyObject) -> c_int;
    #[cfg(Py_3_14)]
    fn PyLong_Sign(v: *mut PyObject) -> c_int;
    // deprecated since 3.14
    #[cfg(all(Py_3_13, not(Py_3_14)))]
    fn _PyLong_Sign(v: *mut PyObject) -> c_int;

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
