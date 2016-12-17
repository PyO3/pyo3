use libc::{c_char, c_int};
use object::PyObject;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyCodec_Register(search_function: *mut PyObject) -> c_int;
    pub fn PyCodec_KnownEncoding(encoding: *const c_char)
     -> c_int;
    pub fn PyCodec_Encode(object: *mut PyObject,
                          encoding: *const c_char,
                          errors: *const c_char) -> *mut PyObject;
    pub fn PyCodec_Decode(object: *mut PyObject,
                          encoding: *const c_char,
                          errors: *const c_char) -> *mut PyObject;
    pub fn PyCodec_Encoder(encoding: *const c_char) -> *mut PyObject;
    pub fn PyCodec_Decoder(encoding: *const c_char) -> *mut PyObject;
    pub fn PyCodec_IncrementalEncoder(encoding: *const c_char,
                                      errors: *const c_char)
     -> *mut PyObject;
    pub fn PyCodec_IncrementalDecoder(encoding: *const c_char,
                                      errors: *const c_char)
     -> *mut PyObject;
    pub fn PyCodec_StreamReader(encoding: *const c_char,
                                stream: *mut PyObject,
                                errors: *const c_char)
     -> *mut PyObject;
    pub fn PyCodec_StreamWriter(encoding: *const c_char,
                                stream: *mut PyObject,
                                errors: *const c_char)
     -> *mut PyObject;
    pub fn PyCodec_RegisterError(name: *const c_char,
                                 error: *mut PyObject) -> c_int;
    pub fn PyCodec_LookupError(name: *const c_char) -> *mut PyObject;
    pub fn PyCodec_StrictErrors(exc: *mut PyObject) -> *mut PyObject;
    pub fn PyCodec_IgnoreErrors(exc: *mut PyObject) -> *mut PyObject;
    pub fn PyCodec_ReplaceErrors(exc: *mut PyObject) -> *mut PyObject;
    pub fn PyCodec_XMLCharRefReplaceErrors(exc: *mut PyObject)
     -> *mut PyObject;
    pub fn PyCodec_BackslashReplaceErrors(exc: *mut PyObject)
     -> *mut PyObject;
}

