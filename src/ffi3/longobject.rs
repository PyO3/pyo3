use libc::size_t;
use std::os::raw::{c_void, c_char, c_int, c_long, c_ulong, c_longlong, c_ulonglong, c_double};
use ffi3::object::*;
use ffi3::pyport::Py_ssize_t;

pub enum PyLongObject {}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyLong_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyLong_Check(op : *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_LONG_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyLong_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyLong_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyLong_FromLong(arg1: c_long) -> *mut PyObject;
    pub fn PyLong_FromUnsignedLong(arg1: c_ulong) -> *mut PyObject;
    pub fn PyLong_FromSize_t(arg1: size_t) -> *mut PyObject;
    pub fn PyLong_FromSsize_t(arg1: Py_ssize_t) -> *mut PyObject;
    pub fn PyLong_FromDouble(arg1: c_double) -> *mut PyObject;
    pub fn PyLong_AsLong(arg1: *mut PyObject) -> c_long;
    pub fn PyLong_AsLongAndOverflow(arg1: *mut PyObject,
                                    arg2: *mut c_int)
     -> c_long;
    pub fn PyLong_AsSsize_t(arg1: *mut PyObject) -> Py_ssize_t;
    pub fn PyLong_AsSize_t(arg1: *mut PyObject) -> size_t;
    pub fn PyLong_AsUnsignedLong(arg1: *mut PyObject) -> c_ulong;
    pub fn PyLong_AsUnsignedLongMask(arg1: *mut PyObject) -> c_ulong;
    pub fn PyLong_GetInfo() -> *mut PyObject;
    pub fn PyLong_AsDouble(arg1: *mut PyObject) -> c_double;
    pub fn PyLong_FromVoidPtr(arg1: *mut c_void) -> *mut PyObject;
    pub fn PyLong_AsVoidPtr(arg1: *mut PyObject) -> *mut c_void;
    pub fn PyLong_FromLongLong(arg1: c_longlong) -> *mut PyObject;
    pub fn PyLong_FromUnsignedLongLong(arg1: c_ulonglong)
     -> *mut PyObject;
    pub fn PyLong_AsLongLong(arg1: *mut PyObject) -> c_longlong;
    pub fn PyLong_AsUnsignedLongLong(arg1: *mut PyObject)
     -> c_ulonglong;
    pub fn PyLong_AsUnsignedLongLongMask(arg1: *mut PyObject)
     -> c_ulonglong;
    pub fn PyLong_AsLongLongAndOverflow(arg1: *mut PyObject,
                                        arg2: *mut c_int)
     -> c_longlong;
    pub fn PyLong_FromString(arg1: *const c_char,
                             arg2: *mut *mut c_char,
                             arg3: c_int) -> *mut PyObject;
    pub fn PyOS_strtoul(arg1: *const c_char,
                        arg2: *mut *mut c_char, arg3: c_int)
     -> c_ulong;
    pub fn PyOS_strtol(arg1: *const c_char,
                       arg2: *mut *mut c_char, arg3: c_int)
     -> c_long;
}

