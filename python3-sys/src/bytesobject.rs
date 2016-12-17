use libc::{c_char, c_int};
use object::*;
use pyport::Py_ssize_t;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyBytes_Type: PyTypeObject;
    pub static mut PyBytesIter_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyBytes_Check(op : *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_BYTES_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyBytes_CheckExact(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyBytes_Type) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyBytes_FromStringAndSize(arg1: *const c_char,
                                     arg2: Py_ssize_t) -> *mut PyObject;
    pub fn PyBytes_FromString(arg1: *const c_char) -> *mut PyObject;
    pub fn PyBytes_FromObject(arg1: *mut PyObject) -> *mut PyObject;
    //pub fn PyBytes_FromFormatV(arg1: *const c_char, arg2: va_list)
    // -> *mut PyObject;
    pub fn PyBytes_FromFormat(arg1: *const c_char, ...)
     -> *mut PyObject;
    pub fn PyBytes_Size(arg1: *mut PyObject) -> Py_ssize_t;
    pub fn PyBytes_AsString(arg1: *mut PyObject) -> *mut c_char;
    pub fn PyBytes_Repr(arg1: *mut PyObject, arg2: c_int)
     -> *mut PyObject;
    pub fn PyBytes_Concat(arg1: *mut *mut PyObject, arg2: *mut PyObject)
     -> ();
    pub fn PyBytes_ConcatAndDel(arg1: *mut *mut PyObject, arg2: *mut PyObject)
     -> ();
    pub fn PyBytes_DecodeEscape(arg1: *const c_char, arg2: Py_ssize_t,
                                arg3: *const c_char, arg4: Py_ssize_t,
                                arg5: *const c_char) -> *mut PyObject;
    pub fn PyBytes_AsStringAndSize(obj: *mut PyObject,
                                   s: *mut *mut c_char,
                                   len: *mut Py_ssize_t) -> c_int;
}

