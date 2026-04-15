use crate::ffi_ptr_ext::FfiPtrExt;
use crate::types::bytes::PyBytesMethods;
use crate::types::PyBytes;
use crate::{ffi, Borrowed, PyResult};
use std::borrow::Cow;
use std::str;

pub(crate) fn to_cow<'a>(string: Borrowed<'a, '_, crate::types::PyString>) -> PyResult<Cow<'a, str>> {
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    {
        return string.to_str().map(Cow::Borrowed);
    }

    #[cfg(not(any(Py_3_10, not(Py_LIMITED_API))))]
    {
        let bytes = unsafe {
            ffi::PyUnicode_AsUTF8String(string.as_ptr())
                .assume_owned_or_err(string.py())?
                .cast_into_unchecked::<PyBytes>()
        };
        Ok(Cow::Owned(
            unsafe { str::from_utf8_unchecked(bytes.as_bytes()) }.to_owned(),
        ))
    }
}

pub(crate) fn to_string_lossy<'a>(string: Borrowed<'a, '_, crate::types::PyString>) -> Cow<'a, str> {
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    if let Ok(value) = string.to_str() {
        return Cow::Borrowed(value);
    }

    let bytes = unsafe {
        ffi::PyUnicode_AsEncodedString(
            string.as_ptr(),
            c"utf-8".as_ptr(),
            c"surrogatepass".as_ptr(),
        )
        .assume_owned(string.py())
        .cast_into_unchecked::<PyBytes>()
    };
    Cow::Owned(String::from_utf8_lossy(bytes.as_bytes()).into_owned())
}
