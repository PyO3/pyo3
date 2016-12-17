use libc::{c_char, c_int, c_long, c_ulong, c_ulonglong, size_t};
use pyport::Py_ssize_t;
use object::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyIntObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub ob_ival: c_long
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyInt_Type: PyTypeObject;
}

#[inline(always)]
pub unsafe fn PyInt_Check(op : *mut PyObject) -> c_int {
    PyType_FastSubclass(Py_TYPE(op), Py_TPFLAGS_INT_SUBCLASS)
}

#[inline(always)]
pub unsafe fn PyInt_CheckExact(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyInt_Type;
    (Py_TYPE(op) == u) as c_int
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyInt_FromString(str: *mut c_char,
                            pend: *mut *mut c_char,
                            base: c_int) -> *mut PyObject;
    #[cfg(py_sys_config="Py_USING_UNICODE")]
    pub fn PyInt_FromUnicode(u: *mut ::unicodeobject::Py_UNICODE, length: Py_ssize_t,
                             base: c_int) -> *mut PyObject;
    pub fn PyInt_FromLong(ival: c_long) -> *mut PyObject;
    pub fn PyInt_FromSize_t(ival: size_t) -> *mut PyObject;
    pub fn PyInt_FromSsize_t(ival: Py_ssize_t) -> *mut PyObject;
    pub fn PyInt_AsLong(io: *mut PyObject) -> c_long;
    pub fn PyInt_AsSsize_t(io: *mut PyObject) -> Py_ssize_t;
    fn _PyInt_AsInt(io: *mut PyObject) -> c_int;
    pub fn PyInt_AsUnsignedLongMask(io: *mut PyObject) -> c_ulong;
    pub fn PyInt_AsUnsignedLongLongMask(io: *mut PyObject)
     -> c_ulonglong;
    pub fn PyInt_GetMax() -> c_long;
    //fn PyOS_strtoul(arg1: *mut c_char,
    //                   arg2: *mut *mut c_char, arg3: c_int)
    // -> c_ulong;
    //fn PyOS_strtol(arg1: *mut c_char,
    //                  arg2: *mut *mut c_char, arg3: c_int)
    // -> c_long;
    pub fn PyInt_ClearFreeList() -> c_int;
    //fn _PyInt_Format(v: *mut PyIntObject, base: c_int,
    //                     newstyle: c_int) -> *mut PyObject;
    //fn _PyInt_FormatAdvanced(obj: *mut PyObject,
    //                             format_spec: *mut c_char,
    //                             format_spec_len: Py_ssize_t)
    // -> *mut PyObject;
}

pub unsafe fn PyInt_AS_LONG(io: *mut PyObject) -> c_long {
    (*(io as *mut PyIntObject)).ob_ival
}

