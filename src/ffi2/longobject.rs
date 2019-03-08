use crate::ffi2::object::*;
use crate::ffi2::pyport::Py_ssize_t;
use libc::size_t;
use std::os::raw::{
    c_char, c_double, c_int, c_long, c_longlong, c_uchar, c_ulong, c_ulonglong, c_void,
};

/// This is an opaque type in the python c api
#[repr(transparent)]
pub struct PyLongObject(*mut c_void);

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyLong_Type")]
    pub static mut PyLong_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyLong_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_LONG_SUBCLASS)
}

#[inline]
pub unsafe fn PyLong_CheckExact(op: *mut PyObject) -> c_int {
    let u: *mut PyTypeObject = &mut PyLong_Type;
    (Py_TYPE(op) == u) as c_int
}

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromLong")]
    pub fn PyLong_FromLong(v: c_long) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromUnsignedLong")]
    pub fn PyLong_FromUnsignedLong(v: c_ulong) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromSsize_t")]
    pub fn PyLong_FromSsize_t(v: Py_ssize_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromSize_t")]
    pub fn PyLong_FromSize_t(v: size_t) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromLongLong")]
    pub fn PyLong_FromLongLong(v: c_longlong) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromUnsignedLongLong")]
    pub fn PyLong_FromUnsignedLongLong(v: c_ulonglong) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromDouble")]
    pub fn PyLong_FromDouble(v: c_double) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromString")]
    pub fn PyLong_FromString(
        str: *mut c_char,
        pend: *mut *mut c_char,
        base: c_int,
    ) -> *mut PyObject;
    #[cfg(py_sys_config = "Py_USING_UNICODE")]
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromUnicode")]
    pub fn PyLong_FromUnicode(
        u: *mut crate::ffi2::unicodeobject::Py_UNICODE,
        length: Py_ssize_t,
        base: c_int,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyLong_FromVoidPtr")]
    pub fn PyLong_FromVoidPtr(p: *mut c_void) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLong")]
    pub fn PyLong_AsLong(pylong: *mut PyObject) -> c_long;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLongAndOverflow")]
    pub fn PyLong_AsLongAndOverflow(pylong: *mut PyObject, overflow: *mut c_int) -> c_long;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLongLongAndOverflow")]
    pub fn PyLong_AsLongLongAndOverflow(pylong: *mut PyObject, overflow: *mut c_int) -> c_longlong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsSsize_t")]
    pub fn PyLong_AsSsize_t(pylong: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLong")]
    pub fn PyLong_AsUnsignedLong(pylong: *mut PyObject) -> c_ulong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsLongLong")]
    pub fn PyLong_AsLongLong(pylong: *mut PyObject) -> c_longlong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLongLong")]
    pub fn PyLong_AsUnsignedLongLong(pylong: *mut PyObject) -> c_ulonglong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLongMask")]
    pub fn PyLong_AsUnsignedLongMask(pylong: *mut PyObject) -> c_ulong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsUnsignedLongLongMask")]
    pub fn PyLong_AsUnsignedLongLongMask(pylong: *mut PyObject) -> c_ulonglong;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsDouble")]
    pub fn PyLong_AsDouble(pylong: *mut PyObject) -> c_double;
    #[cfg_attr(PyPy, link_name = "PyPyLong_AsVoidPtr")]
    pub fn PyLong_AsVoidPtr(pylong: *mut PyObject) -> *mut c_void;

    pub fn PyLong_GetInfo() -> *mut PyObject;

#[cfg_attr(PyPy, link_name="\u{1}__PyPyLong_FromByteArray")]
    pub fn _PyLong_FromByteArray(
        bytes: *const c_uchar,
        n: size_t,
        little_endian: c_int,
        is_signed: c_int,
    ) -> *mut PyObject;

    pub fn _PyLong_AsByteArray(
        v: *mut PyLongObject,
        bytes: *const c_uchar,
        n: size_t,
        little_endian: c_int,
        is_signed: c_int,
    ) -> c_int;
}
