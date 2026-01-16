use crate::object::*;
use crate::pyport::Py_ssize_t;
use std::ffi::{c_char, c_int};
use std::ptr::addr_of_mut;

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyBytes_Type")]
    pub static mut PyBytes_Type: PyTypeObject;
    pub static mut PyBytesIter_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyBytes_Check(op: *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_BYTES_SUBCLASS)
}

#[inline]
pub unsafe fn PyBytes_CheckExact(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyBytes_Type)) as c_int
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyBytes_FromStringAndSize")]
    pub fn PyBytes_FromStringAndSize(arg1: *const c_char, arg2: Py_ssize_t) -> *mut PyObject;
    pub fn PyBytes_FromString(arg1: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyBytes_FromObject")]
    pub fn PyBytes_FromObject(arg1: *mut PyObject) -> *mut PyObject;
    // skipped PyBytes_FromFormatV
    //#[cfg_attr(PyPy, link_name = "PyPyBytes_FromFormatV")]
    //pub fn PyBytes_FromFormatV(arg1: *const c_char, arg2: va_list)
    // -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyBytes_FromFormat")]
    pub fn PyBytes_FromFormat(arg1: *const c_char, ...) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyBytes_Size")]
    pub fn PyBytes_Size(arg1: *mut PyObject) -> Py_ssize_t;
    #[cfg_attr(PyPy, link_name = "PyPyBytes_AsString")]
    pub fn PyBytes_AsString(arg1: *mut PyObject) -> *mut c_char;
    pub fn PyBytes_Repr(arg1: *mut PyObject, arg2: c_int) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyBytes_Concat")]
    pub fn PyBytes_Concat(arg1: *mut *mut PyObject, arg2: *mut PyObject);
    #[cfg_attr(PyPy, link_name = "PyPyBytes_ConcatAndDel")]
    pub fn PyBytes_ConcatAndDel(arg1: *mut *mut PyObject, arg2: *mut PyObject);
    pub fn PyBytes_DecodeEscape(
        arg1: *const c_char,
        arg2: Py_ssize_t,
        arg3: *const c_char,
        arg4: Py_ssize_t,
        arg5: *const c_char,
    ) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPyBytes_AsStringAndSize")]
    pub fn PyBytes_AsStringAndSize(
        obj: *mut PyObject,
        s: *mut *mut c_char,
        len: *mut Py_ssize_t,
    ) -> c_int;
}

// skipped F_LJUST
// skipped F_SIGN
// skipped F_BLANK
// skipped F_ALT
// skipped F_ZERO
