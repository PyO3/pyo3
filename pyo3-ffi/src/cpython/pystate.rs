#[cfg(not(PyPy))]
use crate::PyThreadState;
use crate::{PyFrameObject, PyInterpreterState, PyObject};
use std::ffi::c_int;

// skipped _PyInterpreterState_RequiresIDRef
// skipped _PyInterpreterState_RequireIDRef

pub type Py_tracefunc = unsafe extern "C" fn(
    obj: *mut PyObject,
    frame: *mut PyFrameObject,
    what: c_int,
    arg: *mut PyObject,
) -> c_int;

pub const PyTrace_CALL: c_int = 0;
pub const PyTrace_EXCEPTION: c_int = 1;
pub const PyTrace_LINE: c_int = 2;
pub const PyTrace_RETURN: c_int = 3;
pub const PyTrace_C_CALL: c_int = 4;
pub const PyTrace_C_EXCEPTION: c_int = 5;
pub const PyTrace_C_RETURN: c_int = 6;
pub const PyTrace_OPCODE: c_int = 7;

// skipped Py_MAX_SCRIPT_PATH_SIZE
// skipped _PyRemoteDebuggerSupport

/// Private structure used inline in `PyGenObject`
#[cfg(not(PyPy))]
#[repr(C)]
#[derive(Clone, Copy)]
pub(crate) struct _PyErr_StackItem {
    #[cfg(not(Py_3_11))]
    exc_type: *mut PyObject,
    exc_value: *mut PyObject,
    #[cfg(not(Py_3_11))]
    exc_traceback: *mut PyObject,
    previous_item: *mut _PyErr_StackItem,
}

// skipped _PyStackChunk
// skipped _ts (aka PyThreadState)

extern "C" {

    #[cfg(Py_3_13)]
    pub fn PyThreadState_GetUnchecked() -> *mut PyThreadState;

    // skipped PyThreadState_EnterTracing
    // skipped PyThreadState_LeaveTracing

    #[cfg_attr(PyPy, link_name = "PyPyGILState_Check")]
    pub fn PyGILState_Check() -> c_int;

    // skipped _PyThread_CurrentFrames
    // skipped _PyThread_CurrentExceptions

    #[cfg(not(PyPy))]
    pub fn PyInterpreterState_Main() -> *mut PyInterpreterState;
    #[cfg_attr(PyPy, link_name = "PyPyInterpreterState_Head")]
    pub fn PyInterpreterState_Head() -> *mut PyInterpreterState;
    #[cfg_attr(PyPy, link_name = "PyPyInterpreterState_Next")]
    pub fn PyInterpreterState_Next(interp: *mut PyInterpreterState) -> *mut PyInterpreterState;
    #[cfg(not(PyPy))]
    pub fn PyInterpreterState_ThreadHead(interp: *mut PyInterpreterState) -> *mut PyThreadState;
    #[cfg(not(PyPy))]
    pub fn PyThreadState_Next(tstate: *mut PyThreadState) -> *mut PyThreadState;

    #[cfg_attr(PyPy, link_name = "PyPyThreadState_DeleteCurrent")]
    pub fn PyThreadState_DeleteCurrent();
}

#[cfg(all(Py_3_9, not(any(Py_3_11, PyPy))))]
pub type _PyFrameEvalFunction = extern "C" fn(
    *mut crate::PyThreadState,
    *mut crate::PyFrameObject,
    c_int,
) -> *mut crate::object::PyObject;

#[cfg(all(Py_3_11, not(PyPy)))]
pub type _PyFrameEvalFunction = extern "C" fn(
    *mut crate::PyThreadState,
    *mut crate::_PyInterpreterFrame,
    c_int,
) -> *mut crate::object::PyObject;

#[cfg(all(Py_3_9, not(PyPy)))]
extern "C" {
    /// Get the frame evaluation function.
    pub fn _PyInterpreterState_GetEvalFrameFunc(
        interp: *mut PyInterpreterState,
    ) -> _PyFrameEvalFunction;

    ///Set the frame evaluation function.
    pub fn _PyInterpreterState_SetEvalFrameFunc(
        interp: *mut PyInterpreterState,
        eval_frame: _PyFrameEvalFunction,
    );
}
