use libc::wchar_t;

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(
    Py_3_13,
    deprecated(note = "Deprecated since Python 3.13. Use `libc::wchar_t` instead.")
)]
pub type Py_UNICODE = wchar_t;

pub type Py_UCS4 = u32;
pub type Py_UCS2 = u16;
pub type Py_UCS1 = u8;

pub const Py_UNICODE_REPLACEMENT_CHARACTER: Py_UCS4 = 0xFFFD;

pub use crate::backend::current::unicodeobject::*;
