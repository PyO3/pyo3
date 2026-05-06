use crate::{PyObject, Py_ssize_t};
use std::ffi::{c_int, c_void};

use crate::Py_uintptr_t;

// skipped PyLong_BASE
// skipped PyLong_MASK
// skipped _PyLong_New
// skipped _PyLong_Copy
// skipped _PyLong_FromDigits
// skipped _PyLong_SIGN_MASK
// skipped _PyLong_NON_SIZE_BITS
// skipped PyUnstable_Long_IsCompact
// skipped PyUnstable_Long_CompactValue

#[derive(Copy, Clone)]
#[repr(C)]
pub struct PyLongLayout {
    pub bits_per_digit: u8,
    pub digit_size: u8,
    pub digits_order: i8,
    pub digit_endianness: i8,
}

extern_libpython! {
    pub fn PyLong_GetNativeLayout() -> *const PyLongLayout;
}

#[repr(C)]
pub struct PyLongExport {
    pub value: i64,
    pub negative: u8,
    pub ndigits: Py_ssize_t,
    pub digits: *const c_void,
    pub _reserved: Py_uintptr_t,
}

extern_libpython! {
    pub fn PyLong_Export(obj: *mut PyObject, export_long: *mut PyLongExport) -> c_int;
    pub fn PyLong_FreeExport(export_long: *mut PyLongExport);
}

opaque_struct!(pub PyLongWriter);

extern_libpython! {
    pub fn PyLongWriter_Create(
        negative: c_int,
        ndigits: Py_ssize_t,
        digits: *mut *mut c_void,
    ) -> *mut PyLongWriter;

    pub fn PyLongWriter_Finish(writer: *mut PyLongWriter) -> *mut PyObject;

    pub fn PyLongWriter_Discard(writer: *mut PyLongWriter);
}
