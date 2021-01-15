use crate::ffi::object::{PyObject, PyTypeObject, Py_TYPE};
use std::mem;
use std::os::raw::{c_char, c_int};

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyCFunction_Type")]
    pub static mut PyCFunction_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyCFunction_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyCFunction_Type) as c_int
}

pub type PyCFunction =
    unsafe extern "C" fn(slf: *mut PyObject, args: *mut PyObject) -> *mut PyObject;

#[cfg(all(Py_3_7, not(Py_LIMITED_API)))]
pub type _PyCFunctionFast = unsafe extern "C" fn(
    slf: *mut PyObject,
    args: *mut *mut PyObject,
    nargs: crate::ffi::pyport::Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject;

pub type PyCFunctionWithKeywords = unsafe extern "C" fn(
    slf: *mut PyObject,
    args: *mut PyObject,
    kwds: *mut PyObject,
) -> *mut PyObject;

#[cfg(Py_3_7)]
pub type _PyCFunctionFastWithKeywords = unsafe extern "C" fn(
    slf: *mut PyObject,
    args: *const *mut PyObject,
    nargs: crate::ffi::pyport::Py_ssize_t,
    kwnames: *mut PyObject,
) -> *mut PyObject;

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyCFunction_GetFunction")]
    pub fn PyCFunction_GetFunction(f: *mut PyObject) -> Option<PyCFunction>;
    pub fn PyCFunction_GetSelf(f: *mut PyObject) -> *mut PyObject;
    pub fn PyCFunction_GetFlags(f: *mut PyObject) -> c_int;
    pub fn PyCFunction_Call(
        f: *mut PyObject,
        args: *mut PyObject,
        kwds: *mut PyObject,
    ) -> *mut PyObject;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyMethodDef {
    pub ml_name: *const c_char,
    pub ml_meth: Option<PyCFunction>,
    pub ml_flags: c_int,
    pub ml_doc: *const c_char,
}

pub const PyMethodDef_INIT: PyMethodDef = PyMethodDef {
    ml_name: std::ptr::null(),
    ml_meth: None,
    ml_flags: 0,
    ml_doc: std::ptr::null(),
};

impl Default for PyMethodDef {
    fn default() -> PyMethodDef {
        unsafe { mem::zeroed() }
    }
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyCFunction_NewEx")]
    pub fn PyCFunction_NewEx(
        ml: *mut PyMethodDef,
        slf: *mut PyObject,
        module: *mut PyObject,
    ) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyCFunction_New")]
    pub fn PyCFunction_New(ml: *mut PyMethodDef, slf: *mut PyObject) -> *mut PyObject;
}

/* Flag passed to newmethodobject */
pub const METH_VARARGS: c_int = 0x0001;
pub const METH_KEYWORDS: c_int = 0x0002;
/* METH_NOARGS and METH_O must not be combined with the flags above. */
pub const METH_NOARGS: c_int = 0x0004;
pub const METH_O: c_int = 0x0008;

/* METH_CLASS and METH_STATIC are a little different; these control
the construction of methods for a class.  These cannot be used for
functions in modules. */
pub const METH_CLASS: c_int = 0x0010;
pub const METH_STATIC: c_int = 0x0020;

/* METH_COEXIST allows a method to be entered eventhough a slot has
already filled the entry.  When defined, the flag allows a separate
method, "__contains__" for example, to coexist with a defined
slot like sq_contains. */

pub const METH_COEXIST: c_int = 0x0040;

/* METH_FASTCALL indicates the PEP 590 Vectorcall calling format. It may
be specified alone or with METH_KEYWORDS. */
#[cfg(all(Py_3_7, not(Py_LIMITED_API)))]
pub const METH_FASTCALL: c_int = 0x0080;

extern "C" {
    pub fn PyCFunction_ClearFreeList() -> c_int;
}
