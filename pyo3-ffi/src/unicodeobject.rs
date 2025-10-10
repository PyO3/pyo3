use crate::object::*;
use crate::pyport::Py_ssize_t;
use libc::wchar_t;
use std::ffi::{c_char, c_int, c_void};
#[cfg(not(PyPy))]
use std::ptr::addr_of_mut;

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(
    Py_3_13,
    deprecated(note = "Deprecated since Python 3.13. Use `libc::wchar_t` instead.")
)]
pub type Py_UNICODE = wchar_t;

pub type Py_UCS4 = u32;
pub type Py_UCS2 = u16;
pub type Py_UCS1 = u8;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Type")]
    pub static mut PyUnicode_Type: PyTypeObject;
    pub static mut PyUnicodeIter_Type: PyTypeObject;

    #[cfg(PyPy)]
    #[link_name = "PyPyUnicode_Check"]
    pub fn PyUnicode_Check(op: *mut PyObject) -> c_int;

    #[cfg(PyPy)]
    #[link_name = "PyPyUnicode_CheckExact"]
    pub fn PyUnicode_CheckExact(op: *mut PyObject) -> c_int;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyUnicode_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_UNICODE_SUBCLASS)
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyUnicode_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyUnicode_Type)) as c_int
}

pub const Py_UNICODE_REPLACEMENT_CHARACTER: Py_UCS4 = 0xFFFD;

extern "C" {

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromStringAndSize")]
    pub fn PyUnicode_FromStringAndSize(u: *const c_char, size: Py_ssize_t) -> *mut PyObject;
    pub fn PyUnicode_FromString(u: *const c_char) -> *mut PyObject;

    pub fn PyUnicode_Substring(
        str: *mut PyObject,
        start: Py_ssize_t,
        end: Py_ssize_t,
    ) -> *mut PyObject;
    pub fn PyUnicode_AsUCS4(
        unicode: *mut PyObject,
        buffer: *mut Py_UCS4,
        buflen: Py_ssize_t,
        copy_null: c_int,
    ) -> *mut Py_UCS4;
    pub fn PyUnicode_AsUCS4Copy(unicode: *mut PyObject) -> *mut Py_UCS4;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_GetLength")]
    pub fn PyUnicode_GetLength(unicode: *mut PyObject) -> Py_ssize_t;
    #[cfg(not(Py_3_12))]
    #[deprecated(note = "Removed in Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_GetSize")]
    pub fn PyUnicode_GetSize(unicode: *mut PyObject) -> Py_ssize_t;
    pub fn PyUnicode_ReadChar(unicode: *mut PyObject, index: Py_ssize_t) -> Py_UCS4;
    pub fn PyUnicode_WriteChar(
        unicode: *mut PyObject,
        index: Py_ssize_t,
        character: Py_UCS4,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Resize")]
    pub fn PyUnicode_Resize(unicode: *mut *mut PyObject, length: Py_ssize_t) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromEncodedObject")]
    pub fn PyUnicode_FromEncodedObject(
        obj: *mut PyObject,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromObject")]
    pub fn PyUnicode_FromObject(obj: *mut PyObject) -> *mut PyObject;
    // #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromFormatV")]
    // pub fn PyUnicode_FromFormatV(format: *const c_char, vargs: va_list) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromFormat")]
    pub fn PyUnicode_FromFormat(format: *const c_char, ...) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_InternInPlace")]
    pub fn PyUnicode_InternInPlace(arg1: *mut *mut PyObject);
    #[cfg(not(Py_3_12))]
    #[cfg_attr(Py_3_10, deprecated(note = "Python 3.10"))]
    pub fn PyUnicode_InternImmortal(arg1: *mut *mut PyObject);
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_InternFromString")]
    pub fn PyUnicode_InternFromString(u: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromWideChar")]
    pub fn PyUnicode_FromWideChar(w: *const wchar_t, size: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsWideChar")]
    pub fn PyUnicode_AsWideChar(
        unicode: *mut PyObject,
        w: *mut wchar_t,
        size: Py_ssize_t,
    ) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsWideCharString")]
    pub fn PyUnicode_AsWideCharString(
        unicode: *mut PyObject,
        size: *mut Py_ssize_t,
    ) -> *mut wchar_t;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromOrdinal")]
    pub fn PyUnicode_FromOrdinal(ordinal: c_int) -> *mut PyObject;
    pub fn PyUnicode_ClearFreeList() -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_GetDefaultEncoding")]
    pub fn PyUnicode_GetDefaultEncoding() -> *const c_char;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Decode")]
    pub fn PyUnicode_Decode(
        s: *const c_char,
        size: Py_ssize_t,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyUnicode_AsDecodedObject(
        unicode: *mut PyObject,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyUnicode_AsDecodedUnicode(
        unicode: *mut PyObject,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsEncodedObject")]
    pub fn PyUnicode_AsEncodedObject(
        unicode: *mut PyObject,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsEncodedString")]
    pub fn PyUnicode_AsEncodedString(
        unicode: *mut PyObject,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyUnicode_AsEncodedUnicode(
        unicode: *mut PyObject,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyUnicode_BuildEncodingMap(string: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicode_DecodeUTF7(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyUnicode_DecodeUTF7Stateful(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
        consumed: *mut Py_ssize_t,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_DecodeUTF8")]
    pub fn PyUnicode_DecodeUTF8(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyUnicode_DecodeUTF8Stateful(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
        consumed: *mut Py_ssize_t,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUTF8String")]
    pub fn PyUnicode_AsUTF8String(unicode: *mut PyObject) -> *mut PyObject;
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUTF8AndSize")]
    pub fn PyUnicode_AsUTF8AndSize(unicode: *mut PyObject, size: *mut Py_ssize_t) -> *const c_char;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_DecodeUTF32")]
    pub fn PyUnicode_DecodeUTF32(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
        byteorder: *mut c_int,
    ) -> *mut PyObject;
    pub fn PyUnicode_DecodeUTF32Stateful(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
        byteorder: *mut c_int,
        consumed: *mut Py_ssize_t,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUTF32String")]
    pub fn PyUnicode_AsUTF32String(unicode: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_DecodeUTF16")]
    pub fn PyUnicode_DecodeUTF16(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
        byteorder: *mut c_int,
    ) -> *mut PyObject;
    pub fn PyUnicode_DecodeUTF16Stateful(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
        byteorder: *mut c_int,
        consumed: *mut Py_ssize_t,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUTF16String")]
    pub fn PyUnicode_AsUTF16String(unicode: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicode_DecodeUnicodeEscape(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUnicodeEscapeString")]
    pub fn PyUnicode_AsUnicodeEscapeString(unicode: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicode_DecodeRawUnicodeEscape(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyUnicode_AsRawUnicodeEscapeString(unicode: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_DecodeLatin1")]
    pub fn PyUnicode_DecodeLatin1(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsLatin1String")]
    pub fn PyUnicode_AsLatin1String(unicode: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_DecodeASCII")]
    pub fn PyUnicode_DecodeASCII(
        string: *const c_char,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsASCIIString")]
    pub fn PyUnicode_AsASCIIString(unicode: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicode_DecodeCharmap(
        string: *const c_char,
        length: Py_ssize_t,
        mapping: *mut PyObject,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyUnicode_AsCharmapString(
        unicode: *mut PyObject,
        mapping: *mut PyObject,
    ) -> *mut PyObject;
    pub fn PyUnicode_DecodeLocaleAndSize(
        str: *const c_char,
        len: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;
    pub fn PyUnicode_DecodeLocale(str: *const c_char, errors: *const c_char) -> *mut PyObject;
    pub fn PyUnicode_EncodeLocale(unicode: *mut PyObject, errors: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FSConverter")]
    pub fn PyUnicode_FSConverter(arg1: *mut PyObject, arg2: *mut c_void) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FSDecoder")]
    pub fn PyUnicode_FSDecoder(arg1: *mut PyObject, arg2: *mut c_void) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_DecodeFSDefault")]
    pub fn PyUnicode_DecodeFSDefault(s: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_DecodeFSDefaultAndSize")]
    pub fn PyUnicode_DecodeFSDefaultAndSize(s: *const c_char, size: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_EncodeFSDefault")]
    pub fn PyUnicode_EncodeFSDefault(unicode: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Concat")]
    pub fn PyUnicode_Concat(left: *mut PyObject, right: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicode_Append(pleft: *mut *mut PyObject, right: *mut PyObject);
    pub fn PyUnicode_AppendAndDel(pleft: *mut *mut PyObject, right: *mut PyObject);
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Split")]
    pub fn PyUnicode_Split(
        s: *mut PyObject,
        sep: *mut PyObject,
        maxsplit: Py_ssize_t,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Splitlines")]
    pub fn PyUnicode_Splitlines(s: *mut PyObject, keepends: c_int) -> *mut PyObject;
    pub fn PyUnicode_Partition(s: *mut PyObject, sep: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicode_RPartition(s: *mut PyObject, sep: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicode_RSplit(
        s: *mut PyObject,
        sep: *mut PyObject,
        maxsplit: Py_ssize_t,
    ) -> *mut PyObject;
    pub fn PyUnicode_Translate(
        str: *mut PyObject,
        table: *mut PyObject,
        errors: *const c_char,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Join")]
    pub fn PyUnicode_Join(separator: *mut PyObject, seq: *mut PyObject) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Tailmatch")]
    pub fn PyUnicode_Tailmatch(
        str: *mut PyObject,
        substr: *mut PyObject,
        start: Py_ssize_t,
        end: Py_ssize_t,
        direction: c_int,
    ) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Find")]
    pub fn PyUnicode_Find(
        str: *mut PyObject,
        substr: *mut PyObject,
        start: Py_ssize_t,
        end: Py_ssize_t,
        direction: c_int,
    ) -> Py_ssize_t;
    pub fn PyUnicode_FindChar(
        str: *mut PyObject,
        ch: Py_UCS4,
        start: Py_ssize_t,
        end: Py_ssize_t,
        direction: c_int,
    ) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Count")]
    pub fn PyUnicode_Count(
        str: *mut PyObject,
        substr: *mut PyObject,
        start: Py_ssize_t,
        end: Py_ssize_t,
    ) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Replace")]
    pub fn PyUnicode_Replace(
        str: *mut PyObject,
        substr: *mut PyObject,
        replstr: *mut PyObject,
        maxcount: Py_ssize_t,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Compare")]
    pub fn PyUnicode_Compare(left: *mut PyObject, right: *mut PyObject) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_CompareWithASCIIString")]
    pub fn PyUnicode_CompareWithASCIIString(left: *mut PyObject, right: *const c_char) -> c_int;
    #[cfg(Py_3_13)]
    pub fn PyUnicode_EqualToUTF8(unicode: *mut PyObject, string: *const c_char) -> c_int;
    #[cfg(Py_3_13)]
    pub fn PyUnicode_EqualToUTF8AndSize(
        unicode: *mut PyObject,
        string: *const c_char,
        size: Py_ssize_t,
    ) -> c_int;

    pub fn PyUnicode_RichCompare(
        left: *mut PyObject,
        right: *mut PyObject,
        op: c_int,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_Format")]
    pub fn PyUnicode_Format(format: *mut PyObject, args: *mut PyObject) -> *mut PyObject;
    pub fn PyUnicode_Contains(container: *mut PyObject, element: *mut PyObject) -> c_int;
    pub fn PyUnicode_IsIdentifier(s: *mut PyObject) -> c_int;
}
