use crate::object::*;
use crate::pyport::Py_ssize_t;
use libc::size_t;
use std::ffi::c_void;
use std::ffi::{c_char, c_double, c_int, c_long, c_longlong, c_uchar, c_ulong, c_ulonglong};

opaque_struct!(pub PyLongObject);

#[inline]
pub unsafe fn PyLong_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_LONG_SUBCLASS)
}

#[inline]
pub unsafe fn PyLong_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &raw mut PyLong_Type) as c_int
}

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromLong")]
    pub fn PyLong_FromLong(arg1: c_long) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromUnsignedLong")]
    pub fn PyLong_FromUnsignedLong(arg1: c_ulong) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromSize_t")]
    pub fn PyLong_FromSize_t(arg1: size_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromSsize_t")]
    pub fn PyLong_FromSsize_t(arg1: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromDouble")]
    pub fn PyLong_FromDouble(arg1: c_double) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLong")]
    pub fn PyLong_AsLong(arg1: *mut PyObject) -> c_long;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLongAndOverflow")]
    pub fn PyLong_AsLongAndOverflow(arg1: *mut PyObject, arg2: *mut c_int) -> c_long;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsSsize_t")]
    pub fn PyLong_AsSsize_t(arg1: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsSize_t")]
    pub fn PyLong_AsSize_t(arg1: *mut PyObject) -> size_t;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLong")]
    pub fn PyLong_AsUnsignedLong(arg1: *mut PyObject) -> c_ulong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLongMask")]
    pub fn PyLong_AsUnsignedLongMask(arg1: *mut PyObject) -> c_ulong;
    pub fn PyLong_GetInfo() -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsDouble")]
    pub fn PyLong_AsDouble(arg1: *mut PyObject) -> c_double;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromVoidPtr")]
    pub fn PyLong_FromVoidPtr(arg1: *mut c_void) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsVoidPtr")]
    pub fn PyLong_AsVoidPtr(arg1: *mut PyObject) -> *mut c_void;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromLongLong")]
    pub fn PyLong_FromLongLong(arg1: c_longlong) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromUnsignedLongLong")]
    pub fn PyLong_FromUnsignedLongLong(arg1: c_ulonglong) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLongLong")]
    pub fn PyLong_AsLongLong(arg1: *mut PyObject) -> c_longlong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLongLong")]
    pub fn PyLong_AsUnsignedLongLong(arg1: *mut PyObject) -> c_ulonglong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLongLongMask")]
    pub fn PyLong_AsUnsignedLongLongMask(arg1: *mut PyObject) -> c_ulonglong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLongLongAndOverflow")]
    pub fn PyLong_AsLongLongAndOverflow(arg1: *mut PyObject, arg2: *mut c_int) -> c_longlong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromString")]
    pub fn PyLong_FromString(
        arg1: *const c_char,
        arg2: *mut *mut c_char,
        arg3: c_int,
    ) -> *mut PyObject;
}

#[cfg(not(Py_LIMITED_API))]
extern_libpython! {
    #[cfg_attr(PyPy, link_name = "_PyPyLong_NumBits")]
    pub fn _PyLong_NumBits(obj: *mut PyObject) -> size_t;
}

#[cfg(Py_3_13)]
extern_libpython! {
    pub fn PyLong_FromUnicodeObject(u: *mut PyObject, base: c_int) -> *mut PyObject;
    pub fn PyLong_AsNativeBytes(
        v: *mut PyObject,
        buffer: *mut c_void,
        n_bytes: Py_ssize_t,
        flags: c_int,
    ) -> Py_ssize_t;
    pub fn PyLong_FromNativeBytes(
        buffer: *const c_void,
        n_bytes: size_t,
        flags: c_int,
    ) -> *mut PyObject;
    pub fn PyLong_FromUnsignedNativeBytes(
        buffer: *const c_void,
        n_bytes: size_t,
        flags: c_int,
    ) -> *mut PyObject;
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

#[cfg(not(Py_LIMITED_API))]
extern_libpython! {
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
}

extern_libpython! {
    pub fn PyOS_strtoul(arg1: *const c_char, arg2: *mut *mut c_char, arg3: c_int) -> c_ulong;
    pub fn PyOS_strtol(arg1: *const c_char, arg2: *mut *mut c_char, arg3: c_int) -> c_long;
}
