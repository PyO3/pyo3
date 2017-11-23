// Copyright (c) 2017-present PyO3 Project and Contributors
//
// based on Daniel Grunwald's https://github.com/dgrunwald/rust-cpython

use std::{mem, str, char};
use std::borrow::Cow;

use python::Python;
use err::{PyErr, PyResult};
use objects::exc;

/// Enum of possible Python string representations.
#[derive(Clone, Copy, Debug)]
pub enum PyStringData<'a> {
    Latin1(&'a [u8]),
    Utf8(&'a [u8]),
    Utf16(&'a [u16]),
    Utf32(&'a [u32])
}

impl <'a> From<&'a str> for PyStringData<'a> {
    #[inline]
    fn from(val: &'a str) -> PyStringData<'a> {
    PyStringData::Utf8(val.as_bytes())
}
}

impl <'a> From<&'a [u16]> for PyStringData<'a> {
    #[inline]
    fn from(val: &'a [u16]) -> PyStringData<'a> {
        PyStringData::Utf16(val)
    }
}

impl <'a> From<&'a [u32]> for PyStringData<'a> {
    #[inline]
    fn from(val: &'a [u32]) -> PyStringData<'a> {
        PyStringData::Utf32(val)
    }
}

impl <'a> PyStringData<'a> {
    /// Convert the Python string data to a Rust string.
    ///
    /// For UTF-8 and ASCII-only latin-1, returns a borrow into the original string data.
    /// For Latin-1, UTF-16 and UTF-32, returns an owned string.
    ///
    /// Fails with UnicodeDecodeError if the string data isn't valid in its encoding.
    pub fn to_string(self, py: Python) -> PyResult<Cow<'a, str>> {
        match self {
            PyStringData::Utf8(data) => {
                match str::from_utf8(data) {
                    Ok(s) => Ok(Cow::Borrowed(s)),
                    Err(e) => Err(PyErr::from_instance(
                        exc::UnicodeDecodeError::new_utf8(py, data, e)?))
                }
            }
            PyStringData::Latin1(data) => {
                if data.iter().all(|&b| b.is_ascii()) {
                    Ok(Cow::Borrowed(unsafe { str::from_utf8_unchecked(data) }))
                } else {
                    Ok(Cow::Owned(data.iter().map(|&b| b as char).collect()))
                }
            },
            PyStringData::Utf16(data) => {
                fn utf16_bytes(input: &[u16]) -> &[u8] {
                    unsafe { mem::transmute(input) }
                }
                match String::from_utf16(data) {
                    Ok(s) => Ok(Cow::Owned(s)),
                    Err(_) => Err(PyErr::from_instance(
                        exc::UnicodeDecodeError::new_err(
                            py, cstr!("utf-16"),
                            utf16_bytes(data), 0 .. 2*data.len(), cstr!("invalid utf-16"))?)
                    )
                }
            },
            PyStringData::Utf32(data) => {
                fn utf32_bytes(input: &[u32]) -> &[u8] {
                    unsafe { mem::transmute(input) }
                }
                match data.iter().map(|&u| char::from_u32(u)).collect() {
                    Some(s) => Ok(Cow::Owned(s)),
                    None => Err(PyErr::from_instance(
                        exc::UnicodeDecodeError::new_err(
                            py, cstr!("utf-32"),
                            utf32_bytes(data), 0 .. 4*data.len(), cstr!("invalid utf-32"))?)
                    )
                }
            }
        }
    }

    /// Convert the Python string data to a Rust string.
    ///
    /// Returns a borrow into the original string data if possible.
    ///
    /// Data that isn't valid in its encoding will be replaced
    /// with U+FFFD REPLACEMENT CHARACTER.
    pub fn to_string_lossy(self) -> Cow<'a, str> {
        match self {
            PyStringData::Utf8(data) => String::from_utf8_lossy(data),
            PyStringData::Latin1(data) => {
                if data.iter().all(|&b| b.is_ascii()) {
                    Cow::Borrowed(unsafe { str::from_utf8_unchecked(data) })
                } else {
                    Cow::Owned(data.iter().map(|&b| b as char).collect())
                }
            },
            PyStringData::Utf16(data) => {
                Cow::Owned(String::from_utf16_lossy(data))
            },
            PyStringData::Utf32(data) => {
                Cow::Owned(data.iter()
                           .map(|&u| char::from_u32(u).unwrap_or('\u{FFFD}'))
                           .collect())
            }
        }
    }
}
