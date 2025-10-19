#[cfg(not(GraalPy))]
use crate::cpython::code::PyCodeObject;
#[cfg(not(GraalPy))]
use crate::object::*;
#[cfg(not(GraalPy))]
use crate::pystate::PyThreadState;
use crate::PyFrameObject;
#[cfg(not(any(PyPy, GraalPy, Py_3_11)))]
use std::ffi::c_char;
use std::ffi::c_int;

#[cfg(not(any(PyPy, GraalPy, Py_3_11)))]
pub type PyFrameState = c_char;

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(not(any(PyPy, GraalPy, Py_3_11)))]
pub struct PyTryBlock {
    pub b_type: c_int,
    pub b_handler: c_int,
    pub b_level: c_int,
}

// skipped _PyFrame_IsRunnable
// skipped _PyFrame_IsExecuting
// skipped _PyFrameHasCompleted

extern "C" {
    #[cfg(not(GraalPy))]
    #[cfg_attr(PyPy, link_name = "PyPyFrame_New")]
    pub fn PyFrame_New(
        tstate: *mut PyThreadState,
        code: *mut PyCodeObject,
        globals: *mut PyObject,
        locals: *mut PyObject,
    ) -> *mut PyFrameObject;
    // skipped _PyFrame_New_NoTrack

    pub fn PyFrame_BlockSetup(f: *mut PyFrameObject, _type: c_int, handler: c_int, level: c_int);
    #[cfg(not(any(PyPy, GraalPy, Py_3_11)))]
    pub fn PyFrame_BlockPop(f: *mut PyFrameObject) -> *mut PyTryBlock;

    pub fn PyFrame_LocalsToFast(f: *mut PyFrameObject, clear: c_int);
    pub fn PyFrame_FastToLocalsWithError(f: *mut PyFrameObject) -> c_int;
    pub fn PyFrame_FastToLocals(f: *mut PyFrameObject);

    // skipped _PyFrame_DebugMallocStats

    #[cfg(not(Py_3_9))]
    pub fn PyFrame_ClearFreeList() -> c_int;
}
