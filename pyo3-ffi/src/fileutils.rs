use crate::pyport::Py_ssize_t;
use core::ffi::c_char;
use libc::wchar_t;

extern "C" {
    pub fn Py_DecodeLocale(arg1: *const c_char, size: *mut Py_ssize_t) -> *mut wchar_t;

    pub fn Py_EncodeLocale(text: *const wchar_t, error_pos: *mut Py_ssize_t) -> *mut c_char;
}
