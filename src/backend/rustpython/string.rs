use crate::ffi_ptr_ext::FfiPtrExt;
use crate::types::bytes::PyBytesMethods;
use crate::types::PyBytes;
use crate::{ffi, Borrowed, PyResult};
use std::borrow::Cow;
use std::str;

pub(crate) fn to_cow<'a>(string: Borrowed<'a, '_, crate::types::PyString>) -> PyResult<Cow<'a, str>> {
    let bytes = unsafe {
        ffi::PyUnicode_AsUTF8String(string.as_ptr())
            .assume_owned_or_err(string.py())?
            .cast_into_unchecked::<PyBytes>()
    };
    Ok(Cow::Owned(
        unsafe { str::from_utf8_unchecked(bytes.as_bytes()) }.to_owned(),
    ))
}

pub(crate) fn to_string_lossy<'a>(string: Borrowed<'a, '_, crate::types::PyString>) -> Cow<'a, str> {
    let bytes = unsafe {
        #[cfg(PyRustPython)]
        let owned = ffi::PyUnicode_AsWtf8String(string.as_ptr()).assume_owned(string.py());
        #[cfg(not(PyRustPython))]
        let owned = ffi::PyUnicode_AsUTF8String(string.as_ptr()).assume_owned(string.py());
        owned.cast_into_unchecked::<PyBytes>()
    };
    Cow::Owned(String::from_utf8_lossy(bytes.as_bytes()).into_owned())
}
