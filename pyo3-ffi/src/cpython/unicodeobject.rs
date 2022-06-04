#[cfg(not(PyPy))]
use crate::Py_hash_t;
use crate::{PyObject, Py_UCS1, Py_UCS2, Py_UCS4, Py_UNICODE, Py_ssize_t};
use libc::wchar_t;
use std::os::raw::{c_char, c_int, c_uint, c_void};

// skipped Py_UNICODE_ISSPACE()
// skipped Py_UNICODE_ISLOWER()
// skipped Py_UNICODE_ISUPPER()
// skipped Py_UNICODE_ISTITLE()
// skipped Py_UNICODE_ISLINEBREAK
// skipped Py_UNICODE_TOLOWER
// skipped Py_UNICODE_TOUPPER
// skipped Py_UNICODE_TOTITLE
// skipped Py_UNICODE_ISDECIMAL
// skipped Py_UNICODE_ISDIGIT
// skipped Py_UNICODE_ISNUMERIC
// skipped Py_UNICODE_ISPRINTABLE
// skipped Py_UNICODE_TODECIMAL
// skipped Py_UNICODE_TODIGIT
// skipped Py_UNICODE_TONUMERIC
// skipped Py_UNICODE_ISALPHA
// skipped Py_UNICODE_ISALNUM
// skipped Py_UNICODE_COPY
// skipped Py_UNICODE_FILL
// skipped Py_UNICODE_IS_SURROGATE
// skipped Py_UNICODE_IS_HIGH_SURROGATE
// skipped Py_UNICODE_IS_LOW_SURROGATE
// skipped Py_UNICODE_JOIN_SURROGATES
// skipped Py_UNICODE_HIGH_SURROGATE
// skipped Py_UNICODE_LOW_SURROGATE

#[repr(C)]
pub struct PyASCIIObject {
    pub ob_base: PyObject,
    pub length: Py_ssize_t,
    #[cfg(not(PyPy))]
    pub hash: Py_hash_t,
    /// A bit field with various properties.
    ///
    /// Rust doesn't expose bitfields. So we have accessor functions for
    /// retrieving values.
    ///
    /// unsigned int interned:2; // SSTATE_* constants.
    /// unsigned int kind:3;     // PyUnicode_*_KIND constants.
    /// unsigned int compact:1;
    /// unsigned int ascii:1;
    /// unsigned int ready:1;
    /// unsigned int :24;
    pub state: u32,
    pub wstr: *mut wchar_t,
}

/// Interacting with the bitfield is not actually well-defined, so we mark these APIs unsafe.
///
/// In addition, they are disabled on big-endian architectures to restrict this to most "common"
/// platforms, which are at least tested on CI and appear to be sound.
#[cfg(target_endian = "little")]
impl PyASCIIObject {
    #[inline]
    pub unsafe fn interned(&self) -> c_uint {
        self.state & 3
    }

    #[inline]
    pub unsafe fn kind(&self) -> c_uint {
        (self.state >> 2) & 7
    }

    #[inline]
    pub unsafe fn compact(&self) -> c_uint {
        (self.state >> 5) & 1
    }

    #[inline]
    pub unsafe fn ascii(&self) -> c_uint {
        (self.state >> 6) & 1
    }

    #[inline]
    pub unsafe fn ready(&self) -> c_uint {
        (self.state >> 7) & 1
    }
}

#[repr(C)]
pub struct PyCompactUnicodeObject {
    pub _base: PyASCIIObject,
    pub utf8_length: Py_ssize_t,
    pub utf8: *mut c_char,
    pub wstr_length: Py_ssize_t,
}

#[repr(C)]
pub union PyUnicodeObjectData {
    pub any: *mut c_void,
    pub latin1: *mut Py_UCS1,
    pub ucs2: *mut Py_UCS2,
    pub ucs4: *mut Py_UCS4,
}

#[repr(C)]
pub struct PyUnicodeObject {
    pub _base: PyCompactUnicodeObject,
    pub data: PyUnicodeObjectData,
}

extern "C" {
    #[cfg(not(PyPy))]
    pub fn _PyUnicode_CheckConsistency(op: *mut PyObject, check_content: c_int) -> c_int;
}

// skipped PyUnicode_GET_SIZE
// skipped PyUnicode_GET_DATA_SIZE
// skipped PyUnicode_AS_UNICODE
// skipped PyUnicode_AS_DATA

pub const SSTATE_NOT_INTERNED: c_uint = 0;
pub const SSTATE_INTERNED_MORTAL: c_uint = 1;
pub const SSTATE_INTERNED_IMMORTAL: c_uint = 2;

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_IS_ASCII(op: *mut PyObject) -> c_uint {
    debug_assert!(crate::PyUnicode_Check(op) != 0);
    debug_assert!(PyUnicode_IS_READY(op) != 0);

    (*(op as *mut PyASCIIObject)).ascii()
}

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_IS_COMPACT(op: *mut PyObject) -> c_uint {
    (*(op as *mut PyASCIIObject)).compact()
}

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_IS_COMPACT_ASCII(op: *mut PyObject) -> c_uint {
    if (*(op as *mut PyASCIIObject)).ascii() != 0 && PyUnicode_IS_COMPACT(op) != 0 {
        1
    } else {
        0
    }
}

#[cfg(not(Py_3_12))]
#[cfg_attr(Py_3_10, deprecated(note = "Python 3.10"))]
pub const PyUnicode_WCHAR_KIND: c_uint = 0;

pub const PyUnicode_1BYTE_KIND: c_uint = 1;
pub const PyUnicode_2BYTE_KIND: c_uint = 2;
pub const PyUnicode_4BYTE_KIND: c_uint = 4;

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_1BYTE_DATA(op: *mut PyObject) -> *mut Py_UCS1 {
    PyUnicode_DATA(op) as *mut Py_UCS1
}

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_2BYTE_DATA(op: *mut PyObject) -> *mut Py_UCS2 {
    PyUnicode_DATA(op) as *mut Py_UCS2
}

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_4BYTE_DATA(op: *mut PyObject) -> *mut Py_UCS4 {
    PyUnicode_DATA(op) as *mut Py_UCS4
}

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_KIND(op: *mut PyObject) -> c_uint {
    debug_assert!(crate::PyUnicode_Check(op) != 0);
    debug_assert!(PyUnicode_IS_READY(op) != 0);

    (*(op as *mut PyASCIIObject)).kind()
}

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn _PyUnicode_COMPACT_DATA(op: *mut PyObject) -> *mut c_void {
    if PyUnicode_IS_ASCII(op) != 0 {
        (op as *mut PyASCIIObject).offset(1) as *mut c_void
    } else {
        (op as *mut PyCompactUnicodeObject).offset(1) as *mut c_void
    }
}

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn _PyUnicode_NONCOMPACT_DATA(op: *mut PyObject) -> *mut c_void {
    debug_assert!(!(*(op as *mut PyUnicodeObject)).data.any.is_null());

    (*(op as *mut PyUnicodeObject)).data.any
}

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_DATA(op: *mut PyObject) -> *mut c_void {
    debug_assert!(crate::PyUnicode_Check(op) != 0);

    if PyUnicode_IS_COMPACT(op) != 0 {
        _PyUnicode_COMPACT_DATA(op)
    } else {
        _PyUnicode_NONCOMPACT_DATA(op)
    }
}

// skipped PyUnicode_WRITE
// skipped PyUnicode_READ
// skipped PyUnicode_READ_CHAR

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_GET_LENGTH(op: *mut PyObject) -> Py_ssize_t {
    debug_assert!(crate::PyUnicode_Check(op) != 0);
    debug_assert!(PyUnicode_IS_READY(op) != 0);

    (*(op as *mut PyASCIIObject)).length
}

#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_IS_READY(op: *mut PyObject) -> c_uint {
    (*(op as *mut PyASCIIObject)).ready()
}

#[cfg(not(Py_3_12))]
#[cfg_attr(Py_3_10, deprecated(note = "Python 3.10"))]
#[inline]
#[cfg(target_endian = "little")]
pub unsafe fn PyUnicode_READY(op: *mut PyObject) -> c_int {
    debug_assert!(crate::PyUnicode_Check(op) != 0);

    if PyUnicode_IS_READY(op) != 0 {
        0
    } else {
        _PyUnicode_Ready(op)
    }
}

// skipped PyUnicode_MAX_CHAR_VALUE
// skipped _PyUnicode_get_wstr_length
// skipped PyUnicode_WSTR_LENGTH

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_New")]
    pub fn PyUnicode_New(size: Py_ssize_t, maxchar: Py_UCS4) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "_PyPyUnicode_Ready")]
    pub fn _PyUnicode_Ready(unicode: *mut PyObject) -> c_int;

    // skipped _PyUnicode_Copy

    #[cfg(not(PyPy))]
    pub fn PyUnicode_CopyCharacters(
        to: *mut PyObject,
        to_start: Py_ssize_t,
        from: *mut PyObject,
        from_start: Py_ssize_t,
        how_many: Py_ssize_t,
    ) -> Py_ssize_t;

    // skipped _PyUnicode_FastCopyCharacters

    #[cfg(not(PyPy))]
    pub fn PyUnicode_Fill(
        unicode: *mut PyObject,
        start: Py_ssize_t,
        length: Py_ssize_t,
        fill_char: Py_UCS4,
    ) -> Py_ssize_t;

    // skipped _PyUnicode_FastFill

    #[cfg(not(Py_3_12))]
    #[deprecated]
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromUnicode")]
    pub fn PyUnicode_FromUnicode(u: *const Py_UNICODE, size: Py_ssize_t) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_FromKindAndData")]
    pub fn PyUnicode_FromKindAndData(
        kind: c_int,
        buffer: *const c_void,
        size: Py_ssize_t,
    ) -> *mut PyObject;

    // skipped _PyUnicode_FromASCII
    // skipped _PyUnicode_FindMaxChar

    #[cfg(not(Py_3_12))]
    #[deprecated]
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUnicode")]
    pub fn PyUnicode_AsUnicode(unicode: *mut PyObject) -> *mut Py_UNICODE;

    // skipped _PyUnicode_AsUnicode

    #[cfg(not(Py_3_12))]
    #[deprecated]
    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUnicodeAndSize")]
    pub fn PyUnicode_AsUnicodeAndSize(
        unicode: *mut PyObject,
        size: *mut Py_ssize_t,
    ) -> *mut Py_UNICODE;

    // skipped PyUnicode_GetMax
}

// skipped _PyUnicodeWriter
// skipped _PyUnicodeWriter_Init
// skipped _PyUnicodeWriter_Prepare
// skipped _PyUnicodeWriter_PrepareInternal
// skipped _PyUnicodeWriter_PrepareKind
// skipped _PyUnicodeWriter_PrepareKindInternal
// skipped _PyUnicodeWriter_WriteChar
// skipped _PyUnicodeWriter_WriteStr
// skipped _PyUnicodeWriter_WriteSubstring
// skipped _PyUnicodeWriter_WriteASCIIString
// skipped _PyUnicodeWriter_WriteLatin1String
// skipped _PyUnicodeWriter_Finish
// skipped _PyUnicodeWriter_Dealloc
// skipped _PyUnicode_FormatAdvancedWriter

extern "C" {
    // skipped _PyUnicode_AsStringAndSize

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_AsUTF8")]
    pub fn PyUnicode_AsUTF8(unicode: *mut PyObject) -> *const c_char;

    // skipped _PyUnicode_AsString

    pub fn PyUnicode_Encode(
        s: *const Py_UNICODE,
        size: Py_ssize_t,
        encoding: *const c_char,
        errors: *const c_char,
    ) -> *mut PyObject;

    pub fn PyUnicode_EncodeUTF7(
        data: *const Py_UNICODE,
        length: Py_ssize_t,
        base64SetO: c_int,
        base64WhiteSpace: c_int,
        errors: *const c_char,
    ) -> *mut PyObject;

    // skipped _PyUnicode_EncodeUTF7
    // skipped _PyUnicode_AsUTF8String

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_EncodeUTF8")]
    pub fn PyUnicode_EncodeUTF8(
        data: *const Py_UNICODE,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;

    pub fn PyUnicode_EncodeUTF32(
        data: *const Py_UNICODE,
        length: Py_ssize_t,
        errors: *const c_char,
        byteorder: c_int,
    ) -> *mut PyObject;

    // skipped _PyUnicode_EncodeUTF32

    pub fn PyUnicode_EncodeUTF16(
        data: *const Py_UNICODE,
        length: Py_ssize_t,
        errors: *const c_char,
        byteorder: c_int,
    ) -> *mut PyObject;

    // skipped _PyUnicode_EncodeUTF16
    // skipped _PyUnicode_DecodeUnicodeEscape

    pub fn PyUnicode_EncodeUnicodeEscape(
        data: *const Py_UNICODE,
        length: Py_ssize_t,
    ) -> *mut PyObject;

    pub fn PyUnicode_EncodeRawUnicodeEscape(
        data: *const Py_UNICODE,
        length: Py_ssize_t,
    ) -> *mut PyObject;

    // skipped _PyUnicode_AsLatin1String

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_EncodeLatin1")]
    pub fn PyUnicode_EncodeLatin1(
        data: *const Py_UNICODE,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;

    // skipped _PyUnicode_AsASCIIString

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_EncodeASCII")]
    pub fn PyUnicode_EncodeASCII(
        data: *const Py_UNICODE,
        length: Py_ssize_t,
        errors: *const c_char,
    ) -> *mut PyObject;

    pub fn PyUnicode_EncodeCharmap(
        data: *const Py_UNICODE,
        length: Py_ssize_t,
        mapping: *mut PyObject,
        errors: *const c_char,
    ) -> *mut PyObject;

    // skipped _PyUnicode_EncodeCharmap

    pub fn PyUnicode_TranslateCharmap(
        data: *const Py_UNICODE,
        length: Py_ssize_t,
        table: *mut PyObject,
        errors: *const c_char,
    ) -> *mut PyObject;

    // skipped PyUnicode_EncodeMBCS

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_EncodeDecimal")]
    pub fn PyUnicode_EncodeDecimal(
        s: *mut Py_UNICODE,
        length: Py_ssize_t,
        output: *mut c_char,
        errors: *const c_char,
    ) -> c_int;

    #[cfg_attr(PyPy, link_name = "PyPyUnicode_TransformDecimalToASCII")]
    pub fn PyUnicode_TransformDecimalToASCII(
        s: *mut Py_UNICODE,
        length: Py_ssize_t,
    ) -> *mut PyObject;

    // skipped _PyUnicode_TransformDecimalAndSpaceToASCII
}

// skipped _PyUnicode_JoinArray
// skipped _PyUnicode_EqualToASCIIId
// skipped _PyUnicode_EqualToASCIIString
// skipped _PyUnicode_XStrip
// skipped _PyUnicode_InsertThousandsGrouping

// skipped _Py_ascii_whitespace

// skipped _PyUnicode_IsLowercase
// skipped _PyUnicode_IsUppercase
// skipped _PyUnicode_IsTitlecase
// skipped _PyUnicode_IsXidStart
// skipped _PyUnicode_IsXidContinue
// skipped _PyUnicode_IsWhitespace
// skipped _PyUnicode_IsLinebreak
// skipped _PyUnicode_ToLowercase
// skipped _PyUnicode_ToUppercase
// skipped _PyUnicode_ToTitlecase
// skipped _PyUnicode_ToLowerFull
// skipped _PyUnicode_ToTitleFull
// skipped _PyUnicode_ToUpperFull
// skipped _PyUnicode_ToFoldedFull
// skipped _PyUnicode_IsCaseIgnorable
// skipped _PyUnicode_IsCased
// skipped _PyUnicode_ToDecimalDigit
// skipped _PyUnicode_ToDigit
// skipped _PyUnicode_ToNumeric
// skipped _PyUnicode_IsDecimalDigit
// skipped _PyUnicode_IsDigit
// skipped _PyUnicode_IsNumeric
// skipped _PyUnicode_IsPrintable
// skipped _PyUnicode_IsAlpha
// skipped Py_UNICODE_strlen
// skipped Py_UNICODE_strcpy
// skipped Py_UNICODE_strcat
// skipped Py_UNICODE_strncpy
// skipped Py_UNICODE_strcmp
// skipped Py_UNICODE_strncmp
// skipped Py_UNICODE_strchr
// skipped Py_UNICODE_strrchr
// skipped _PyUnicode_FormatLong
// skipped PyUnicode_AsUnicodeCopy
// skipped _PyUnicode_FromId
// skipped _PyUnicode_EQ
// skipped _PyUnicode_ScanIdentifier
