use crate::pyport::Py_ssize_t;
use libc::wchar_t;
use std::ffi::c_char;

extern "C" {
    pub fn Py_DecodeLocale(arg1: *const c_char, size: *mut Py_ssize_t) -> *mut wchar_t;

    pub fn Py_EncodeLocale(text: *const wchar_t, error_pos: *mut Py_ssize_t) -> *mut c_char;
}
