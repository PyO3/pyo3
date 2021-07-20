use crate::ffi::pystate::{PyInterpreterState, PyThreadState};
use std::os::raw::c_int;

// Py_tracefunc is defined in ffi::pystate

pub const PyTrace_CALL: c_int = 0;
pub const PyTrace_EXCEPTION: c_int = 1;
pub const PyTrace_LINE: c_int = 2;
pub const PyTrace_RETURN: c_int = 3;
pub const PyTrace_C_CALL: c_int = 4;
pub const PyTrace_C_EXCEPTION: c_int = 5;
pub const PyTrace_C_RETURN: c_int = 6;
pub const PyTrace_OPCODE: c_int = 7;

extern "C" {
    // PyGILState_Check is defined in ffi::pystate
    pub fn PyInterpreterState_Main() -> *mut PyInterpreterState;
    pub fn PyInterpreterState_Head() -> *mut PyInterpreterState;
    pub fn PyInterpreterState_Next(interp: *mut PyInterpreterState) -> *mut PyInterpreterState;
    pub fn PyInterpreterState_ThreadHead(interp: *mut PyInterpreterState) -> *mut PyThreadState;
    pub fn PyThreadState_Next(tstate: *mut PyThreadState) -> *mut PyThreadState;
}

#[cfg(Py_3_9)]
#[cfg_attr(docsrs, doc(cfg(Py_3_9)))]
pub type _PyFrameEvalFunction = extern "C" fn(
    *mut crate::ffi::PyThreadState,
    *mut crate::ffi::PyFrameObject,
    c_int,
) -> *mut crate::ffi::object::PyObject;

#[cfg(Py_3_9)]
extern "C" {
    /// Get the frame evaluation function.
    #[cfg_attr(docsrs, doc(cfg(Py_3_9)))]
    pub fn _PyInterpreterState_GetEvalFrameFunc(
        interp: *mut PyInterpreterState,
    ) -> _PyFrameEvalFunction;

    ///Set the frame evaluation function.
    #[cfg_attr(docsrs, doc(cfg(Py_3_9)))]
    pub fn _PyInterpreterState_SetEvalFrameFunc(
        interp: *mut PyInterpreterState,
        eval_frame: _PyFrameEvalFunction,
    );
}
