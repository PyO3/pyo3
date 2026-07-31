use crate::object::*;
use crate::pyport::Py_ssize_t;
#[cfg(any(all(Py_3_14, not(Py_LIMITED_API)), Py_3_15))]
use crate::Py_uintptr_t;
use core::ffi::{c_char, c_double, c_int, c_long, c_longlong, c_ulong, c_ulonglong, c_void};
use libc::size_t;

opaque_struct!(pub PyLongObject);

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyLong_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_LONG_SUBCLASS)
}

#[inline]
#[cfg(not(RustPython))]
pub unsafe fn PyLong_CheckExact(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyLong_Type)
}

extern_libpython! {
    #[cfg(RustPython)]
    pub fn PyLong_Check(op: *mut PyObject) -> c_int;
    #[cfg(RustPython)]
    pub fn PyLong_CheckExact(op: *mut PyObject) -> c_int;

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

    // skipped non-limited PyLong_AsInt

    #[cfg(Py_3_14)]
    pub fn PyLong_FromInt32(arg1: i32) -> *mut PyObject;
    #[cfg(Py_3_14)]
    pub fn PyLong_FromUInt32(arg1: u32) -> *mut PyObject;
    #[cfg(Py_3_14)]
    pub fn PyLong_FromInt64(arg1: i64) -> *mut PyObject;
    #[cfg(Py_3_14)]
    pub fn PyLong_FromUInt64(arg1: u64) -> *mut PyObject;

    #[cfg(Py_3_14)]
    pub fn PyLong_AsInt32(arg1: *mut PyObject, arg2: *mut i32) -> c_int;
    #[cfg(Py_3_14)]
    pub fn PyLong_AsUInt32(arg1: *mut PyObject, arg2: *mut u32) -> c_int;
    #[cfg(Py_3_14)]
    pub fn PyLong_AsInt64(arg1: *mut PyObject, arg2: *mut i64) -> c_int;
    #[cfg(Py_3_14)]
    pub fn PyLong_AsUInt64(arg1: *mut PyObject, arg2: *mut u64) -> c_int;
}

#[cfg(any(Py_3_14, all(Py_3_13, not(Py_LIMITED_API))))]
pub const Py_ASNATIVEBYTES_DEFAULTS: c_int = -1;
#[cfg(any(Py_3_14, all(Py_3_13, not(Py_LIMITED_API))))]
pub const Py_ASNATIVEBYTES_BIG_ENDIAN: c_int = 0;
#[cfg(any(Py_3_14, all(Py_3_13, not(Py_LIMITED_API))))]
pub const Py_ASNATIVEBYTES_LITTLE_ENDIAN: c_int = 1;
#[cfg(any(Py_3_14, all(Py_3_13, not(Py_LIMITED_API))))]
pub const Py_ASNATIVEBYTES_NATIVE_ENDIAN: c_int = 3;
#[cfg(any(Py_3_14, all(Py_3_13, not(Py_LIMITED_API))))]
pub const Py_ASNATIVEBYTES_UNSIGNED_BUFFER: c_int = 4;
#[cfg(any(Py_3_14, all(Py_3_13, not(Py_LIMITED_API))))]
pub const Py_ASNATIVEBYTES_REJECT_NEGATIVE: c_int = 8;
#[cfg(any(Py_3_14, all(Py_3_13, not(Py_LIMITED_API))))]
pub const Py_ASNATIVEBYTES_ALLOW_INDEX: c_int = 16;

extern_libpython! {
    #[cfg(any(Py_3_14, all(Py_3_13, not(Py_LIMITED_API))))]
    pub fn PyLong_AsNativeBytes(
        v: *mut PyObject,
        buffer: *mut c_void,
        n_bytes: Py_ssize_t,
        flags: c_int,
    ) -> Py_ssize_t;

    #[cfg(any(Py_3_14, all(Py_3_13, not(Py_LIMITED_API))))]
    pub fn PyLong_FromNativeBytes(
        buffer: *const c_void,
        n_bytes: size_t,
        flags: c_int,
    ) -> *mut PyObject;

    #[cfg(any(Py_3_14, all(Py_3_13, not(Py_LIMITED_API))))]
    pub fn PyLong_FromUnsignedNativeBytes(
        buffer: *const c_void,
        n_bytes: size_t,
        flags: c_int,
    ) -> *mut PyObject;

    pub fn PyLong_GetInfo() -> *mut PyObject;
    // skipped PyLong_AS_LONG

    // skipped PyLong_FromPid
    // skipped PyLong_AsPid
    // skipped _Py_PARSE_INTPTR
    // skipped _Py_PARSE_UINTPTR

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

extern_libpython! {
    pub fn PyOS_strtoul(arg1: *const c_char, arg2: *mut *mut c_char, arg3: c_int) -> c_ulong;
    pub fn PyOS_strtol(arg1: *const c_char, arg2: *mut *mut c_char, arg3: c_int) -> c_long;
}

#[cfg(any(all(Py_3_14, not(Py_LIMITED_API)), Py_3_15))]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct PyLongLayout {
    pub bits_per_digit: u8,
    pub digit_size: u8,
    pub digits_order: i8,
    pub digit_endianness: i8,
}

#[cfg(any(all(Py_3_14, not(Py_LIMITED_API)), Py_3_15))]
extern_libpython! {
    pub fn PyLong_GetNativeLayout() -> *const PyLongLayout;
}

#[cfg(any(all(Py_3_14, not(Py_LIMITED_API)), Py_3_15))]
#[repr(C)]
pub struct PyLongExport {
    pub value: i64,
    pub negative: u8,
    pub ndigits: Py_ssize_t,
    pub digits: *const c_void,
    _reserved: Py_uintptr_t,
}

#[cfg(any(all(Py_3_14, not(Py_LIMITED_API)), Py_3_15))]
extern_libpython! {
    pub fn PyLong_Export(obj: *mut PyObject, export_long: *mut PyLongExport) -> c_int;
    pub fn PyLong_FreeExport(export_long: *mut PyLongExport);
}

#[cfg(any(all(Py_3_14, not(Py_LIMITED_API)), Py_3_15))]
opaque_struct!(pub PyLongWriter);

#[cfg(any(all(Py_3_14, not(Py_LIMITED_API)), Py_3_15))]
extern_libpython! {
    pub fn PyLongWriter_Create(
        negative: c_int,
        ndigits: Py_ssize_t,
        digits: *mut *mut c_void,
    ) -> *mut PyLongWriter;

    pub fn PyLongWriter_Finish(writer: *mut PyLongWriter) -> *mut PyObject;

    pub fn PyLongWriter_Discard(writer: *mut PyLongWriter);
}
