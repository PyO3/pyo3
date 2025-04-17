#[allow(unused_imports)]
use crate::object::PyObject;
#[cfg(not(GraalPy))]
#[cfg(any(Py_3_10, all(Py_3_9, not(Py_LIMITED_API))))]
use crate::PyCodeObject;
#[cfg(not(Py_LIMITED_API))]
use crate::PyFrameObject;
#[allow(unused_imports)]
use std::os::raw::{c_char, c_int};

#[cfg(Py_LIMITED_API)]
opaque_struct!(PyFrameObject);

// skipped _PyInterpreterFrame from Include/cpython/pyframe.h

extern "C" {
    pub fn PyFrame_GetLineNumber(frame: *mut PyFrameObject) -> c_int;

    #[cfg(Py_3_9)]
    pub fn PyFrame_GetBack(frame: *mut PyFrameObject) -> *mut PyFrameObject;

    #[cfg(not(GraalPy))]
    #[cfg(any(Py_3_10, all(Py_3_9, not(Py_LIMITED_API))))]
    pub fn PyFrame_GetCode(frame: *mut PyFrameObject) -> *mut PyCodeObject;

    #[cfg(Py_3_11)]
    pub fn PyFrame_GetGenerator(frame: *mut PyFrameObject) -> *mut PyObject;

    #[cfg(Py_3_11)]
    pub fn PyFrame_GetBuiltins(frame: *mut PyFrameObject) -> *mut PyObject;

    #[cfg(Py_3_11)]
    pub fn PyFrame_GetLocals(frame: *mut PyFrameObject) -> *mut PyObject;

    #[cfg(Py_3_11)]
    pub fn PyFrame_GetGlobals(frame: *mut PyFrameObject) -> *mut PyObject;

    #[cfg(Py_3_11)]
    pub fn PyFrame_GetLasti(frame: *mut PyFrameObject) -> c_int;

    #[cfg(Py_3_12)]
    pub fn PyFrame_GetVar(frame: *mut PyFrameObject, name: *mut PyObject) -> *mut PyObject;

    #[cfg(Py_3_12)]
    pub fn PyFrame_GetVarString(frame: *mut PyFrameObject, name: *mut c_char) -> *mut PyObject;

    // skipped PyUnstable_InterpreterFrame_GetCode from Include/cpython/pyframe.h
    // skipped PyUnstable_InterpreterFrame_GetLasti from Include/cpython/pyframe.h
    // skipped PyUnstable_InterpreterFrame_GetLine from Include/cpython/pyframe.h
}
