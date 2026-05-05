use crate::longobject::*;
use crate::object::*;
#[cfg(Py_3_13)]
use crate::pyport::Py_ssize_t;
#[cfg(Py_3_14)]
use crate::Py_uintptr_t;
use libc::size_t;
#[cfg(Py_3_13)]
use std::ffi::c_void;
use std::ffi::{c_int, c_uchar};

#[cfg(Py_3_13)]
extern_libpython! {
    pub fn PyLong_FromUnicodeObject(u: *mut PyObject, base: c_int) -> *mut PyObject;
}

#[cfg(Py_3_13)]
pub const Py_ASNATIVEBYTES_DEFAULTS: c_int = -1;
#[cfg(Py_3_13)]
pub const Py_ASNATIVEBYTES_BIG_ENDIAN: c_int = 0;
#[cfg(Py_3_13)]
pub const Py_ASNATIVEBYTES_LITTLE_ENDIAN: c_int = 1;
#[cfg(Py_3_13)]
pub const Py_ASNATIVEBYTES_NATIVE_ENDIAN: c_int = 3;
#[cfg(Py_3_13)]
pub const Py_ASNATIVEBYTES_UNSIGNED_BUFFER: c_int = 4;
#[cfg(Py_3_13)]
pub const Py_ASNATIVEBYTES_REJECT_NEGATIVE: c_int = 8;

#[cfg(Py_3_14)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PyLongLayout {
    pub bits_per_digit: u8,
    pub digit_size: u8,
    pub digits_order: i8,
    pub digit_endianness: i8,
}

#[cfg(Py_3_14)]
#[repr(C)]
pub struct PyLongExport {
    pub value: i64,
    pub negative: u8,
    pub ndigits: Py_ssize_t,
    pub digits: *const c_void,
    pub _reserved: Py_uintptr_t,
}

#[cfg(Py_3_14)]
opaque_struct!(pub PyLongWriter);

extern_libpython! {
    // skipped _PyLong_Sign

    #[cfg(Py_3_13)]
    pub fn PyLong_AsNativeBytes(
        v: *mut PyObject,
        buffer: *mut c_void,
        n_bytes: Py_ssize_t,
        flags: c_int,
    ) -> Py_ssize_t;

    #[cfg(Py_3_13)]
    pub fn PyLong_FromNativeBytes(
        buffer: *const c_void,
        n_bytes: size_t,
        flags: c_int,
    ) -> *mut PyObject;

    #[cfg(Py_3_13)]
    pub fn PyLong_FromUnsignedNativeBytes(
        buffer: *const c_void,
        n_bytes: size_t,
        flags: c_int,
    ) -> *mut PyObject;

    // skipped PyUnstable_Long_IsCompact
    // skipped PyUnstable_Long_CompactValue

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

    #[cfg(Py_3_14)]
    pub fn PyLong_GetNativeLayout() -> *const PyLongLayout;

    #[cfg(Py_3_14)]
    pub fn PyLong_Export(obj: *mut PyObject, export_long: *mut PyLongExport) -> c_int;

    #[cfg(Py_3_14)]
    pub fn PyLong_FreeExport(export_long: *mut PyLongExport);

    #[cfg(Py_3_14)]
    pub fn PyLongWriter_Create(
        negative: c_int,
        ndigits: Py_ssize_t,
        digits: *mut *mut c_void,
    ) -> *mut PyLongWriter;

    #[cfg(Py_3_14)]
    pub fn PyLongWriter_Finish(writer: *mut PyLongWriter) -> *mut PyObject;

    #[cfg(Py_3_14)]
    pub fn PyLongWriter_Discard(writer: *mut PyLongWriter);
}
