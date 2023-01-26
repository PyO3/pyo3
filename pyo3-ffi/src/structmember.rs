use crate::object::PyObject;
use crate::pyport::Py_ssize_t;
use std::os::raw::{c_char, c_int};
use std::ptr;

/// Represents the [PyMemberDef](https://docs.python.org/3/c-api/structures.html#c.PyMemberDef)
/// structure.
///
/// Note that CPython may leave fields uninitialized. You must always ensure that
/// `name` != NULL before dereferencing or reading other fields.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyMemberDef {
    pub name: *mut c_char,
    pub type_code: c_int,
    pub offset: Py_ssize_t,
    pub flags: c_int,
    pub doc: *mut c_char,
}

impl Default for PyMemberDef {
    fn default() -> PyMemberDef {
        PyMemberDef {
            name: ptr::null_mut(),
            type_code: 0,
            offset: 0,
            flags: 0,
            doc: ptr::null_mut(),
        }
    }
}

/* Types */
pub const T_SHORT: c_int = 0;
pub const T_INT: c_int = 1;
pub const T_LONG: c_int = 2;
pub const T_FLOAT: c_int = 3;
pub const T_DOUBLE: c_int = 4;
pub const T_STRING: c_int = 5;
pub const T_OBJECT: c_int = 6;
/* XXX the ordering here is weird for binary compatibility */
pub const T_CHAR: c_int = 7; /* 1-character string */
pub const T_BYTE: c_int = 8; /* 8-bit signed int */
/* unsigned variants: */
pub const T_UBYTE: c_int = 9;
pub const T_USHORT: c_int = 10;
pub const T_UINT: c_int = 11;
pub const T_ULONG: c_int = 12;

/* Added by Jack: strings contained in the structure */
pub const T_STRING_INPLACE: c_int = 13;

/* Added by Lillo: bools contained in the structure (assumed char) */
pub const T_BOOL: c_int = 14;

pub const T_OBJECT_EX: c_int = 16; /* Like T_OBJECT, but raises AttributeError
                                   when the value is NULL, instead of
                                   converting to None. */

pub const T_LONGLONG: c_int = 17;
pub const T_ULONGLONG: c_int = 18;

pub const T_PYSSIZET: c_int = 19; /* Py_ssize_t */
pub const T_NONE: c_int = 20; /* Value is always None */

/* Flags */
pub const READONLY: c_int = 1;
pub const READ_RESTRICTED: c_int = 2;
pub const PY_WRITE_RESTRICTED: c_int = 4;
pub const RESTRICTED: c_int = READ_RESTRICTED | PY_WRITE_RESTRICTED;

extern "C" {
    pub fn PyMember_GetOne(addr: *const c_char, l: *mut PyMemberDef) -> *mut PyObject;
    pub fn PyMember_SetOne(addr: *mut c_char, l: *mut PyMemberDef, value: *mut PyObject) -> c_int;
}
