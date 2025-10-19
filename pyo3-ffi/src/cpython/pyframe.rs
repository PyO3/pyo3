#[cfg(any(Py_3_11, all(Py_3_9, not(PyPy))))]
use crate::PyFrameObject;
use crate::{PyObject, PyTypeObject, Py_TYPE};
#[cfg(Py_3_12)]
use std::ffi::c_char;
use std::ffi::c_int;
use std::ptr::addr_of_mut;

// NB used in `_PyEval_EvalFrameDefault`, maybe we remove this too.
#[cfg(all(Py_3_11, not(PyPy)))]
opaque_struct!(pub _PyInterpreterFrame);

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyFrame_Type: PyTypeObject;

    #[cfg(Py_3_13)]
    pub static mut PyFrameLocalsProxy_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyFrame_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyFrame_Type)) as c_int
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyFrameLocalsProxy_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyFrameLocalsProxy_Type)) as c_int
}

extern "C" {
    #[cfg(all(Py_3_9, not(PyPy)))]
    pub fn PyFrame_GetBack(frame: *mut PyFrameObject) -> *mut PyFrameObject;

    #[cfg(Py_3_11)]
    pub fn PyFrame_GetLocals(frame: *mut PyFrameObject) -> *mut PyObject;

    #[cfg(Py_3_11)]
    pub fn PyFrame_GetGlobals(frame: *mut PyFrameObject) -> *mut PyObject;

    #[cfg(Py_3_11)]
    pub fn PyFrame_GetBuiltins(frame: *mut PyFrameObject) -> *mut PyObject;

    #[cfg(Py_3_11)]
    pub fn PyFrame_GetGenerator(frame: *mut PyFrameObject) -> *mut PyObject;

    #[cfg(Py_3_11)]
    pub fn PyFrame_GetLasti(frame: *mut PyFrameObject) -> c_int;

    #[cfg(Py_3_12)]
    pub fn PyFrame_GetVar(frame: *mut PyFrameObject, name: *mut PyObject) -> *mut PyObject;

    #[cfg(Py_3_12)]
    pub fn PyFrame_GetVarString(frame: *mut PyFrameObject, name: *mut c_char) -> *mut PyObject;

    // skipped PyUnstable_InterpreterFrame_GetCode
    // skipped PyUnstable_InterpreterFrame_GetLasti
    // skipped PyUnstable_InterpreterFrame_GetLine
    // skipped PyUnstable_ExecutableKinds

}
