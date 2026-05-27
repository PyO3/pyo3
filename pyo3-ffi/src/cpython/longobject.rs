#[cfg(not(Py_3_13))]
use crate::longobject::*;
use crate::object::*;

use core::ffi::c_int;

#[cfg(not(Py_3_13))]
use core::ffi::c_uchar;

#[cfg(not(Py_3_13))]
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

    // skipped PyLong_GetSign
    // skipped _PyLong_Sign
    // skipped non-limited _PyLong_NumBits

    #[cfg(not(Py_3_13))]
    #[cfg_attr(PyPy, link_name = "_PyPyLong_FromByteArray")]
    #[doc(hidden)] // used in PyO3's older bytes conversions, but not otherwise public API
    pub fn _PyLong_FromByteArray(
        bytes: *const c_uchar,
        n: size_t,
        little_endian: c_int,
        is_signed: c_int,
    ) -> *mut PyObject;

    #[cfg(not(Py_3_13))]
    #[doc(hidden)] // used in PyO3's older bytes conversions, but not otherwise public API
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
