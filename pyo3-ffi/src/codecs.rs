use crate::object::PyObject;
use std::ffi::{c_char, c_int};

extern "C" {
    pub fn PyCodec_Register(search_function: *mut PyObject) -> c_int;
    #[cfg(Py_3_10)]
    #[cfg(not(PyPy))]
    pub fn PyCodec_Unregister(search_function: *mut PyObject) -> c_int;
    // skipped non-limited _PyCodec_Lookup from Include/codecs.h
    // skipped non-limited _PyCodec_Forget from Include/codecs.h
    pub fn PyCodec_KnownEncoding(encoding: *const c_char) -> c_int;
    pub fn PyCodec_Encode(
        object: *mut PyObject,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyCodec_Decode(
        object: *mut PyObject,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    // skipped non-limited _PyCodec_LookupTextEncoding from Include/codecs.h
    // skipped non-limited _PyCodec_EncodeText from Include/codecs.h
    // skipped non-limited _PyCodec_DecodeText from Include/codecs.h
    // skipped non-limited _PyCodecInfo_GetIncrementalDecoder from Include/codecs.h
    // skipped non-limited _PyCodecInfo_GetIncrementalEncoder from Include/codecs.h
    pub fn PyCodec_Encoder(encoding: *const c_char) -> *mut PyObject;
    pub fn PyCodec_Decoder(encoding: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyCodec_IncrementalEncoder")]
    pub fn PyCodec_IncrementalEncoder(
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyCodec_IncrementalDecoder")]
    pub fn PyCodec_IncrementalDecoder(
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyCodec_StreamReader(
        encoding: *const c_char,
        stream: *mut PyObject,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyCodec_StreamWriter(
        encoding: *const c_char,
        stream: *mut PyObject,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyCodec_RegisterError(name: *const c_char, error: *mut PyObject) -> c_int;
    pub fn PyCodec_LookupError(name: *const c_char) -> *mut PyObject;
    pub fn PyCodec_StrictErrors(exc: *mut PyObject) -> *mut PyObject;
    pub fn PyCodec_IgnoreErrors(exc: *mut PyObject) -> *mut PyObject;
    pub fn PyCodec_ReplaceErrors(exc: *mut PyObject) -> *mut PyObject;
    pub fn PyCodec_XMLCharRefReplaceErrors(exc: *mut PyObject) -> *mut PyObject;
    pub fn PyCodec_BackslashReplaceErrors(exc: *mut PyObject) -> *mut PyObject;
    // skipped non-limited PyCodec_NameReplaceErrors from Include/codecs.h
    // skipped non-limited Py_hexdigits from Include/codecs.h
}
