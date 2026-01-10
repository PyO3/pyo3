use crate::PyThreadState;
use crate::{PyFrameObject, PyInterpreterState, PyObject};
use std::ffi::c_int;

// skipped private _PyInterpreterState_RequiresIDRef
// skipped private _PyInterpreterState_RequireIDRef

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

// skipped private _Py_MAX_SCRIPT_PATH_SIZE
// skipped private _PyRemoteDebuggerSupport

/// Private structure used inline in `PyGenObject`
///
/// `PyGenObject` was made opaque in Python 3.14, so we don't bother defining this
/// structure for that version and later.
#[cfg(not(any(PyPy, Py_3_14)))]
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

// skipped private _PyStackChunk

// skipped private _PY_DATA_STACK_CHUNK_SIZE
// skipped private _ts (aka PyThreadState)

extern "C" {
    #[cfg(Py_3_13)]
    pub fn PyThreadState_GetUnchecked() -> *mut PyThreadState;

    #[cfg(not(Py_3_13))]
    pub(crate) fn _PyThreadState_UncheckedGet() -> *mut PyThreadState;

    #[cfg(Py_3_11)]
    pub fn PyThreadState_EnterTracing(state: *mut PyThreadState);
    #[cfg(Py_3_11)]
    pub fn PyThreadState_LeaveTracing(state: *mut PyThreadState);

    #[cfg_attr(PyPy, link_name = "PyPyGILState_Check")]
    pub fn PyGILState_Check() -> c_int;

    // skipped private _PyThread_CurrentFrames

    // skipped PyUnstable_ThreadState_SetStackProtection
    // skipped PyUnstable_ThreadState_ResetStackProtection

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

// skipped private _PyFrameEvalFunction
// skipped private _PyInterpreterState_GetEvalFrameFunc
// skipped private _PyInterpreterState_SetEvalFrameFunc
