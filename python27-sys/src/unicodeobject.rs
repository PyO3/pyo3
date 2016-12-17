use libc::{c_char, c_int, c_long, c_double, wchar_t};
use pyport::Py_ssize_t;
use object::*;

#[cfg(py_sys_config="Py_UNICODE_SIZE_4")]
pub const Py_UNICODE_SIZE : Py_ssize_t = 4;
#[cfg(not(py_sys_config="Py_UNICODE_SIZE_4"))]
pub const Py_UNICODE_SIZE : Py_ssize_t = 2;

pub type Py_UCS4 = u32;

#[cfg(py_sys_config="Py_UNICODE_SIZE_4")]
pub type Py_UNICODE = u32;
#[cfg(not(py_sys_config="Py_UNICODE_SIZE_4"))]
pub type Py_UNICODE = u16;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyUnicodeObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub length: Py_ssize_t,
    pub data: *mut Py_UNICODE,
    pub hash: c_long,
    pub defenc: *mut PyObject,
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyUnicode_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyUnicode_Check(op : *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_UNICODE_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyUnicode_CheckExact(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyUnicode_Type;
    (Py_TYPE(op) == u) as c_int
}

#[inline(always)]
pub unsafe fn PyUnicode_GET_SIZE(o: *mut PyObject) -> Py_ssize_t {
    (*(o as *mut PyUnicodeObject)).length
}

#[inline(always)]
pub unsafe fn PyUnicode_GET_DATA_SIZE(o: *mut PyObject) -> Py_ssize_t {
    (*(o as *mut PyUnicodeObject)).length * Py_UNICODE_SIZE
}

#[inline(always)]
pub unsafe fn PyUnicode_AS_UNICODE(o: *mut PyObject) -> *mut Py_UNICODE {
    (*(o as *mut PyUnicodeObject)).data
}

#[inline(always)]
pub unsafe fn PyUnicode_AS_DATA(o: *mut PyObject) -> *const c_char {
    (*(o as *mut PyUnicodeObject)).data as *const c_char
}

pub const Py_UNICODE_REPLACEMENT_CHARACTER : Py_UNICODE = 0xFFFD;


#[allow(dead_code)]
#[cfg(py_sys_config="Py_UNICODE_SIZE_4")]
#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    fn PyUnicodeUCS4_FromUnicode(u: *const Py_UNICODE, size: Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS4_FromStringAndSize(u: *const c_char,
                                           size: Py_ssize_t) -> *mut PyObject;
    fn PyUnicodeUCS4_FromString(u: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsUnicode(unicode: *mut PyObject) -> *mut Py_UNICODE;
    fn PyUnicodeUCS4_GetSize(unicode: *mut PyObject) -> Py_ssize_t;
    fn PyUnicodeUCS4_GetMax() -> Py_UNICODE;
    fn PyUnicodeUCS4_Resize(unicode: *mut *mut PyObject,
                                length: Py_ssize_t) -> c_int;
    fn PyUnicodeUCS4_FromEncodedObject(obj: *mut PyObject,
                                           encoding: *const c_char,
                                           errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_FromObject(obj: *mut PyObject) -> *mut PyObject;
    fn PyUnicodeUCS4_FromFormatV(arg1: *const c_char,
                                     ...) -> *mut PyObject;
    fn PyUnicodeUCS4_FromFormat(arg1: *const c_char, ...)
     -> *mut PyObject;
    fn _PyUnicode_FormatAdvanced(obj: *mut PyObject,
                                     format_spec: *mut Py_UNICODE,
                                     format_spec_len: Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS4_FromWideChar(w: *const wchar_t, size: Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsWideChar(unicode: *mut PyUnicodeObject,
                                    w: *mut wchar_t, size: Py_ssize_t)
     -> Py_ssize_t;
    fn PyUnicodeUCS4_FromOrdinal(ordinal: c_int) -> *mut PyObject;
    fn PyUnicodeUCS4_ClearFreelist() -> c_int;
    fn _PyUnicodeUCS4_AsDefaultEncodedString(arg1: *mut PyObject,
                                                 arg2: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_GetDefaultEncoding() -> *const c_char;
    fn PyUnicodeUCS4_SetDefaultEncoding(encoding: *const c_char)
     -> c_int;
    fn PyUnicodeUCS4_Decode(s: *const c_char, size: Py_ssize_t,
                                encoding: *const c_char,
                                errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_Encode(s: *const Py_UNICODE, size: Py_ssize_t,
                                encoding: *const c_char,
                                errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsEncodedObject(unicode: *mut PyObject,
                                         encoding: *const c_char,
                                         errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsEncodedString(unicode: *mut PyObject,
                                         encoding: *const c_char,
                                         errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicode_BuildEncodingMap(string: *mut PyObject) -> *mut PyObject;
    fn PyUnicode_DecodeUTF7(string: *const c_char,
                                length: Py_ssize_t,
                                errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicode_DecodeUTF7Stateful(string: *const c_char,
                                        length: Py_ssize_t,
                                        errors: *const c_char,
                                        consumed: *mut Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicode_EncodeUTF7(data: *const Py_UNICODE, length: Py_ssize_t,
                                base64SetO: c_int,
                                base64WhiteSpace: c_int,
                                errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeUTF8(string: *const c_char,
                                    length: Py_ssize_t,
                                    errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeUTF8Stateful(string: *const c_char,
                                            length: Py_ssize_t,
                                            errors: *const c_char,
                                            consumed: *mut Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsUTF8String(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_EncodeUTF8(data: *const Py_UNICODE,
                                    length: Py_ssize_t,
                                    errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeUTF32(string: *const c_char,
                                     length: Py_ssize_t,
                                     errors: *const c_char,
                                     byteorder: *mut c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeUTF32Stateful(string: *const c_char,
                                             length: Py_ssize_t,
                                             errors: *const c_char,
                                             byteorder: *mut c_int,
                                             consumed: *mut Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsUTF32String(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_EncodeUTF32(data: *const Py_UNICODE,
                                     length: Py_ssize_t,
                                     errors: *const c_char,
                                     byteorder: c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeUTF16(string: *const c_char,
                                     length: Py_ssize_t,
                                     errors: *const c_char,
                                     byteorder: *mut c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeUTF16Stateful(string: *const c_char,
                                             length: Py_ssize_t,
                                             errors: *const c_char,
                                             byteorder: *mut c_int,
                                             consumed: *mut Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsUTF16String(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_EncodeUTF16(data: *const Py_UNICODE,
                                     length: Py_ssize_t,
                                     errors: *const c_char,
                                     byteorder: c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeUnicodeEscape(string: *const c_char,
                                             length: Py_ssize_t,
                                             errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsUnicodeEscapeString(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_EncodeUnicodeEscape(data: *const Py_UNICODE,
                                             length: Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeRawUnicodeEscape(string: *const c_char,
                                                length: Py_ssize_t,
                                                errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsRawUnicodeEscapeString(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_EncodeRawUnicodeEscape(data: *const Py_UNICODE,
                                                length: Py_ssize_t)
     -> *mut PyObject;
    fn _PyUnicode_DecodeUnicodeInternal(string: *const c_char,
                                            length: Py_ssize_t,
                                            errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeLatin1(string: *const c_char,
                                      length: Py_ssize_t,
                                      errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsLatin1String(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_EncodeLatin1(data: *const Py_UNICODE,
                                      length: Py_ssize_t,
                                      errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeASCII(string: *const c_char,
                                     length: Py_ssize_t,
                                     errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsASCIIString(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_EncodeASCII(data: *const Py_UNICODE,
                                     length: Py_ssize_t,
                                     errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_DecodeCharmap(string: *const c_char,
                                       length: Py_ssize_t,
                                       mapping: *mut PyObject,
                                       errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_AsCharmapString(unicode: *mut PyObject,
                                         mapping: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_EncodeCharmap(data: *const Py_UNICODE,
                                       length: Py_ssize_t,
                                       mapping: *mut PyObject,
                                       errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_TranslateCharmap(data: *const Py_UNICODE,
                                          length: Py_ssize_t,
                                          table: *mut PyObject,
                                          errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_EncodeDecimal(s: *mut Py_UNICODE, length: Py_ssize_t,
                                       output: *mut c_char,
                                       errors: *const c_char)
     -> c_int;
    fn PyUnicodeUCS4_Concat(left: *mut PyObject, right: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_Split(s: *mut PyObject, sep: *mut PyObject,
                               maxsplit: Py_ssize_t) -> *mut PyObject;
    fn PyUnicodeUCS4_Splitlines(s: *mut PyObject, keepends: c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS4_Partition(s: *mut PyObject, sep: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_RPartition(s: *mut PyObject, sep: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_RSplit(s: *mut PyObject, sep: *mut PyObject,
                                maxsplit: Py_ssize_t) -> *mut PyObject;
    fn PyUnicodeUCS4_Translate(str: *mut PyObject, table: *mut PyObject,
                                   errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS4_Join(separator: *mut PyObject, seq: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_Tailmatch(str: *mut PyObject, substr: *mut PyObject,
                                   start: Py_ssize_t, end: Py_ssize_t,
                                   direction: c_int) -> Py_ssize_t;
    fn PyUnicodeUCS4_Find(str: *mut PyObject, substr: *mut PyObject,
                              start: Py_ssize_t, end: Py_ssize_t,
                              direction: c_int) -> Py_ssize_t;
    fn PyUnicodeUCS4_Count(str: *mut PyObject, substr: *mut PyObject,
                               start: Py_ssize_t, end: Py_ssize_t)
     -> Py_ssize_t;
    fn PyUnicodeUCS4_Replace(str: *mut PyObject, substr: *mut PyObject,
                                 replstr: *mut PyObject, maxcount: Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS4_Compare(left: *mut PyObject, right: *mut PyObject)
     -> c_int;
    fn PyUnicodeUCS4_RichCompare(left: *mut PyObject,
                                     right: *mut PyObject, op: c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS4_Format(format: *mut PyObject, args: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS4_Contains(container: *mut PyObject,
                                  element: *mut PyObject) -> c_int;
    fn _PyUnicode_XStrip(_self: *mut PyUnicodeObject,
                             striptype: c_int, sepobj: *mut PyObject)
     -> *mut PyObject;
    fn _PyUnicodeUCS4_IsLowercase(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS4_IsUppercase(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS4_IsTitlecase(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS4_IsWhitespace(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS4_IsLinebreak(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS4_ToLowercase(ch: Py_UNICODE) -> Py_UNICODE;
    fn _PyUnicodeUCS4_ToUppercase(ch: Py_UNICODE) -> Py_UNICODE;
    fn _PyUnicodeUCS4_ToTitlecase(ch: Py_UNICODE) -> Py_UNICODE;
    fn _PyUnicodeUCS4_ToDecimalDigit(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS4_ToDigit(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS4_ToNumeric(ch: Py_UNICODE) -> c_double;
    fn _PyUnicodeUCS4_IsDecimalDigit(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS4_IsDigit(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS4_IsNumeric(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS4_IsAlpha(ch: Py_UNICODE) -> c_int;
}

#[allow(dead_code)]
#[cfg(not(py_sys_config="Py_UNICODE_SIZE_4"))]
#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    fn PyUnicodeUCS2_FromUnicode(u: *const Py_UNICODE, size: Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS2_FromStringAndSize(u: *const c_char,
                                           size: Py_ssize_t) -> *mut PyObject;
    fn PyUnicodeUCS2_FromString(u: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsUnicode(unicode: *mut PyObject) -> *mut Py_UNICODE;
    fn PyUnicodeUCS2_GetSize(unicode: *mut PyObject) -> Py_ssize_t;
    fn PyUnicodeUCS2_GetMax() -> Py_UNICODE;
    fn PyUnicodeUCS2_Resize(unicode: *mut *mut PyObject,
                                length: Py_ssize_t) -> c_int;
    fn PyUnicodeUCS2_FromEncodedObject(obj: *mut PyObject,
                                           encoding: *const c_char,
                                           errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_FromObject(obj: *mut PyObject) -> *mut PyObject;
    fn PyUnicodeUCS2_FromFormatV(arg1: *const c_char,
                                     ...) -> *mut PyObject;
    fn PyUnicodeUCS2_FromFormat(arg1: *const c_char, ...)
     -> *mut PyObject;
    fn _PyUnicode_FormatAdvanced(obj: *mut PyObject,
                                     format_spec: *mut Py_UNICODE,
                                     format_spec_len: Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS2_FromWideChar(w: *const wchar_t, size: Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsWideChar(unicode: *mut PyUnicodeObject,
                                    w: *mut wchar_t, size: Py_ssize_t)
     -> Py_ssize_t;
    fn PyUnicodeUCS2_FromOrdinal(ordinal: c_int) -> *mut PyObject;
    fn PyUnicodeUCS2_ClearFreelist() -> c_int;
    fn _PyUnicodeUCS2_AsDefaultEncodedString(arg1: *mut PyObject,
                                                 arg2: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_GetDefaultEncoding() -> *const c_char;
    fn PyUnicodeUCS2_SetDefaultEncoding(encoding: *const c_char)
     -> c_int;
    fn PyUnicodeUCS2_Decode(s: *const c_char, size: Py_ssize_t,
                                encoding: *const c_char,
                                errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_Encode(s: *const Py_UNICODE, size: Py_ssize_t,
                                encoding: *const c_char,
                                errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsEncodedObject(unicode: *mut PyObject,
                                         encoding: *const c_char,
                                         errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsEncodedString(unicode: *mut PyObject,
                                         encoding: *const c_char,
                                         errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicode_BuildEncodingMap(string: *mut PyObject) -> *mut PyObject;
    fn PyUnicode_DecodeUTF7(string: *const c_char,
                                length: Py_ssize_t,
                                errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicode_DecodeUTF7Stateful(string: *const c_char,
                                        length: Py_ssize_t,
                                        errors: *const c_char,
                                        consumed: *mut Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicode_EncodeUTF7(data: *const Py_UNICODE, length: Py_ssize_t,
                                base64SetO: c_int,
                                base64WhiteSpace: c_int,
                                errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeUTF8(string: *const c_char,
                                    length: Py_ssize_t,
                                    errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeUTF8Stateful(string: *const c_char,
                                            length: Py_ssize_t,
                                            errors: *const c_char,
                                            consumed: *mut Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsUTF8String(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_EncodeUTF8(data: *const Py_UNICODE,
                                    length: Py_ssize_t,
                                    errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeUTF32(string: *const c_char,
                                     length: Py_ssize_t,
                                     errors: *const c_char,
                                     byteorder: *mut c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeUTF32Stateful(string: *const c_char,
                                             length: Py_ssize_t,
                                             errors: *const c_char,
                                             byteorder: *mut c_int,
                                             consumed: *mut Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsUTF32String(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_EncodeUTF32(data: *const Py_UNICODE,
                                     length: Py_ssize_t,
                                     errors: *const c_char,
                                     byteorder: c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeUTF16(string: *const c_char,
                                     length: Py_ssize_t,
                                     errors: *const c_char,
                                     byteorder: *mut c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeUTF16Stateful(string: *const c_char,
                                             length: Py_ssize_t,
                                             errors: *const c_char,
                                             byteorder: *mut c_int,
                                             consumed: *mut Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsUTF16String(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_EncodeUTF16(data: *const Py_UNICODE,
                                     length: Py_ssize_t,
                                     errors: *const c_char,
                                     byteorder: c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeUnicodeEscape(string: *const c_char,
                                             length: Py_ssize_t,
                                             errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsUnicodeEscapeString(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_EncodeUnicodeEscape(data: *const Py_UNICODE,
                                             length: Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeRawUnicodeEscape(string: *const c_char,
                                                length: Py_ssize_t,
                                                errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsRawUnicodeEscapeString(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_EncodeRawUnicodeEscape(data: *const Py_UNICODE,
                                                length: Py_ssize_t)
     -> *mut PyObject;
    fn _PyUnicode_DecodeUnicodeInternal(string: *const c_char,
                                            length: Py_ssize_t,
                                            errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeLatin1(string: *const c_char,
                                      length: Py_ssize_t,
                                      errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsLatin1String(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_EncodeLatin1(data: *const Py_UNICODE,
                                      length: Py_ssize_t,
                                      errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeASCII(string: *const c_char,
                                     length: Py_ssize_t,
                                     errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsASCIIString(unicode: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_EncodeASCII(data: *const Py_UNICODE,
                                     length: Py_ssize_t,
                                     errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_DecodeCharmap(string: *const c_char,
                                       length: Py_ssize_t,
                                       mapping: *mut PyObject,
                                       errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_AsCharmapString(unicode: *mut PyObject,
                                         mapping: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_EncodeCharmap(data: *const Py_UNICODE,
                                       length: Py_ssize_t,
                                       mapping: *mut PyObject,
                                       errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_TranslateCharmap(data: *const Py_UNICODE,
                                          length: Py_ssize_t,
                                          table: *mut PyObject,
                                          errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_EncodeDecimal(s: *mut Py_UNICODE, length: Py_ssize_t,
                                       output: *mut c_char,
                                       errors: *const c_char)
     -> c_int;
    fn PyUnicodeUCS2_Concat(left: *mut PyObject, right: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_Split(s: *mut PyObject, sep: *mut PyObject,
                               maxsplit: Py_ssize_t) -> *mut PyObject;
    fn PyUnicodeUCS2_Splitlines(s: *mut PyObject, keepends: c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS2_Partition(s: *mut PyObject, sep: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_RPartition(s: *mut PyObject, sep: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_RSplit(s: *mut PyObject, sep: *mut PyObject,
                                maxsplit: Py_ssize_t) -> *mut PyObject;
    fn PyUnicodeUCS2_Translate(str: *mut PyObject, table: *mut PyObject,
                                   errors: *const c_char)
     -> *mut PyObject;
    fn PyUnicodeUCS2_Join(separator: *mut PyObject, seq: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_Tailmatch(str: *mut PyObject, substr: *mut PyObject,
                                   start: Py_ssize_t, end: Py_ssize_t,
                                   direction: c_int) -> Py_ssize_t;
    fn PyUnicodeUCS2_Find(str: *mut PyObject, substr: *mut PyObject,
                              start: Py_ssize_t, end: Py_ssize_t,
                              direction: c_int) -> Py_ssize_t;
    fn PyUnicodeUCS2_Count(str: *mut PyObject, substr: *mut PyObject,
                               start: Py_ssize_t, end: Py_ssize_t)
     -> Py_ssize_t;
    fn PyUnicodeUCS2_Replace(str: *mut PyObject, substr: *mut PyObject,
                                 replstr: *mut PyObject, maxcount: Py_ssize_t)
     -> *mut PyObject;
    fn PyUnicodeUCS2_Compare(left: *mut PyObject, right: *mut PyObject)
     -> c_int;
    fn PyUnicodeUCS2_RichCompare(left: *mut PyObject,
                                     right: *mut PyObject, op: c_int)
     -> *mut PyObject;
    fn PyUnicodeUCS2_Format(format: *mut PyObject, args: *mut PyObject)
     -> *mut PyObject;
    fn PyUnicodeUCS2_Contains(container: *mut PyObject,
                                  element: *mut PyObject) -> c_int;
    fn _PyUnicode_XStrip(_self: *mut PyUnicodeObject,
                             striptype: c_int, sepobj: *mut PyObject)
     -> *mut PyObject;
    fn _PyUnicodeUCS2_IsLowercase(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS2_IsUppercase(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS2_IsTitlecase(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS2_IsWhitespace(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS2_IsLinebreak(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS2_ToLowercase(ch: Py_UNICODE) -> Py_UNICODE;
    fn _PyUnicodeUCS2_ToUppercase(ch: Py_UNICODE) -> Py_UNICODE;
    fn _PyUnicodeUCS2_ToTitlecase(ch: Py_UNICODE) -> Py_UNICODE;
    fn _PyUnicodeUCS2_ToDecimalDigit(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS2_ToDigit(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS2_ToNumeric(ch: Py_UNICODE) -> c_double;
    fn _PyUnicodeUCS2_IsDecimalDigit(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS2_IsDigit(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS2_IsNumeric(ch: Py_UNICODE) -> c_int;
    fn _PyUnicodeUCS2_IsAlpha(ch: Py_UNICODE) -> c_int;
}

#[inline(always)]
#[cfg(py_sys_config="Py_UNICODE_SIZE_4")]
pub unsafe fn PyUnicode_FromStringAndSize(u: *const c_char, size: Py_ssize_t) -> *mut PyObject {
    PyUnicodeUCS4_FromStringAndSize(u, size)
}

#[inline(always)]
#[cfg(not(py_sys_config="Py_UNICODE_SIZE_4"))]
pub unsafe fn PyUnicode_FromStringAndSize(u: *const c_char, size: Py_ssize_t) -> *mut PyObject {
    PyUnicodeUCS2_FromStringAndSize(u, size)
}

#[inline(always)]
#[cfg(py_sys_config="Py_UNICODE_SIZE_4")]
pub unsafe fn PyUnicode_AsUTF8String(u: *mut PyObject) -> *mut PyObject {
    PyUnicodeUCS4_AsUTF8String(u)
}

#[inline(always)]
#[cfg(not(py_sys_config="Py_UNICODE_SIZE_4"))]
pub unsafe fn PyUnicode_AsUTF8String(u: *mut PyObject) -> *mut PyObject {
    PyUnicodeUCS2_AsUTF8String(u)
}

