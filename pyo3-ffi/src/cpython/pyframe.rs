#[cfg(any(Py_3_11, all(Py_3_9, not(PyPy))))]
use crate::PyFrameObject;
use crate::{PyObject, PyTypeObject, Py_IS_TYPE};
#[cfg(Py_3_12)]
use std::ffi::c_char;
use std::ffi::c_int;

// NB used in `_PyEval_EvalFrameDefault`, maybe we remove this too.
#[cfg(all(Py_3_11, not(PyPy)))]
opaque_struct!(pub _PyInterpreterFrame);

#[cfg(Py_3_13)]
pub const PyUnstable_EXECUTABLE_KIND_SKIP: c_int = 0;
#[cfg(Py_3_13)]
pub const PyUnstable_EXECUTABLE_KIND_PY_FUNCTION: c_int = 1;
#[cfg(Py_3_13)]
pub const PyUnstable_EXECUTABLE_KIND_BUILTIN_FUNCTION: c_int = 3;
#[cfg(Py_3_13)]
pub const PyUnstable_EXECUTABLE_KIND_METHOD_DESCRIPTOR: c_int = 4;
#[cfg(Py_3_13)]
pub const PyUnstable_EXECUTABLE_KINDS: c_int = 5;

extern_libpython! {
    pub static mut PyFrame_Type: PyTypeObject;

    #[cfg(Py_3_13)]
    pub static mut PyFrameLocalsProxy_Type: PyTypeObject;

    #[cfg(Py_3_13)]
    pub static PyUnstable_ExecutableKinds: [*const PyTypeObject; 6];
}

#[inline]
pub unsafe fn PyFrame_Check(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyFrame_Type)
}

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn PyFrameLocalsProxy_Check(op: *mut PyObject) -> c_int {
    Py_IS_TYPE(op, &raw mut PyFrameLocalsProxy_Type)
}

extern_libpython! {
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

    #[cfg(Py_3_12)]
    pub fn PyUnstable_InterpreterFrame_GetCode(frame: *mut _PyInterpreterFrame) -> *mut PyObject;

    #[cfg(Py_3_12)]
    pub fn PyUnstable_InterpreterFrame_GetLasti(frame: *mut _PyInterpreterFrame) -> *mut PyObject;

    #[cfg(Py_3_12)]
    pub fn PyUnstable_InterpreterFrame_GetLine(frame: *mut _PyInterpreterFrame) -> *mut PyObject;
}
