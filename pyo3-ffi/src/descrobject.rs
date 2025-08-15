use crate::methodobject::PyMethodDef;
use crate::object::{PyObject, PyTypeObject};
use crate::Py_ssize_t;
use std::ffi::{c_char, c_int, c_void};
use std::ptr;

pub type getter = unsafe extern "C" fn(slf: *mut PyObject, closure: *mut c_void) -> *mut PyObject;
pub type setter =
    unsafe extern "C" fn(slf: *mut PyObject, value: *mut PyObject, closure: *mut c_void) -> c_int;

/// Represents the [PyGetSetDef](https://docs.python.org/3/c-api/structures.html#c.PyGetSetDef)
/// structure.
///
/// Note that CPython may leave fields uninitialized. You must ensure that
/// `name` != NULL before dereferencing or reading other fields.
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PyGetSetDef {
    pub name: *const c_char,
    pub get: Option<getter>,
    pub set: Option<setter>,
    pub doc: *const c_char,
    pub closure: *mut c_void,
}

impl Default for PyGetSetDef {
    fn default() -> PyGetSetDef {
        PyGetSetDef {
            name: ptr::null(),
            get: None,
            set: None,
            doc: ptr::null(),
            closure: ptr::null_mut(),
        }
    }
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyClassMethodDescr_Type")]
    pub static mut PyClassMethodDescr_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyGetSetDescr_Type")]
    pub static mut PyGetSetDescr_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyMemberDescr_Type")]
    pub static mut PyMemberDescr_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyMethodDescr_Type")]
    pub static mut PyMethodDescr_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyWrapperDescr_Type")]
    pub static mut PyWrapperDescr_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyDictProxy_Type")]
    pub static mut PyDictProxy_Type: PyTypeObject;
    #[cfg_attr(PyPy, link_name = "PyPyProperty_Type")]
    pub static mut PyProperty_Type: PyTypeObject;
}

extern "C" {
    pub fn PyDescr_NewMethod(arg1: *mut PyTypeObject, arg2: *mut PyMethodDef) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDescr_NewClassMethod")]
    pub fn PyDescr_NewClassMethod(arg1: *mut PyTypeObject, arg2: *mut PyMethodDef)
        -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDescr_NewMember")]
    pub fn PyDescr_NewMember(arg1: *mut PyTypeObject, arg2: *mut PyMemberDef) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyDescr_NewGetSet")]
    pub fn PyDescr_NewGetSet(arg1: *mut PyTypeObject, arg2: *mut PyGetSetDef) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyDictProxy_New")]
    pub fn PyDictProxy_New(arg1: *mut PyObject) -> *mut PyObject;
    pub fn PyWrapper_New(arg1: *mut PyObject, arg2: *mut PyObject) -> *mut PyObject;
}

/// Represents the [PyMemberDef](https://docs.python.org/3/c-api/structures.html#c.PyMemberDef)
/// structure.
///
/// Note that CPython may leave fields uninitialized. You must always ensure that
/// `name` != NULL before dereferencing or reading other fields.
#[repr(C)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct PyMemberDef {
    pub name: *const c_char,
    pub type_code: c_int,
    pub offset: Py_ssize_t,
    pub flags: c_int,
    pub doc: *const c_char,
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
pub const Py_T_SHORT: c_int = 0;
pub const Py_T_INT: c_int = 1;
pub const Py_T_LONG: c_int = 2;
pub const Py_T_FLOAT: c_int = 3;
pub const Py_T_DOUBLE: c_int = 4;
pub const Py_T_STRING: c_int = 5;
#[deprecated(note = "Use Py_T_OBJECT_EX instead")]
pub const _Py_T_OBJECT: c_int = 6;
pub const Py_T_CHAR: c_int = 7;
pub const Py_T_BYTE: c_int = 8;
pub const Py_T_UBYTE: c_int = 9;
pub const Py_T_USHORT: c_int = 10;
pub const Py_T_UINT: c_int = 11;
pub const Py_T_ULONG: c_int = 12;
pub const Py_T_STRING_INPLACE: c_int = 13;
pub const Py_T_BOOL: c_int = 14;
pub const Py_T_OBJECT_EX: c_int = 16;
pub const Py_T_LONGLONG: c_int = 17;
pub const Py_T_ULONGLONG: c_int = 18;
pub const Py_T_PYSSIZET: c_int = 19;
#[deprecated(note = "Value is always none")]
pub const _Py_T_NONE: c_int = 20;

/* Flags */
pub const Py_READONLY: c_int = 1;
#[cfg(Py_3_10)]
pub const Py_AUDIT_READ: c_int = 2; // Added in 3.10, harmless no-op before that
#[deprecated]
pub const _Py_WRITE_RESTRICTED: c_int = 4; // Deprecated, no-op. Do not reuse the value.
pub const Py_RELATIVE_OFFSET: c_int = 8;

extern "C" {
    pub fn PyMember_GetOne(addr: *const c_char, l: *mut PyMemberDef) -> *mut PyObject;
    pub fn PyMember_SetOne(addr: *mut c_char, l: *mut PyMemberDef, value: *mut PyObject) -> c_int;
}
