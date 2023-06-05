use crate::cpython::code::PyCodeObject;
use crate::object::*;
use crate::pystate::PyThreadState;
#[cfg(not(any(PyPy, Py_3_11)))]
use std::os::raw::c_char;
use std::os::raw::c_int;
use std::ptr::addr_of_mut;

#[cfg(not(any(PyPy, Py_3_11)))]
pub type PyFrameState = c_char;

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(not(any(PyPy, Py_3_11)))]
pub struct PyTryBlock {
    pub b_type: c_int,
    pub b_handler: c_int,
    pub b_level: c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(not(any(PyPy, Py_3_11)))]
pub struct PyFrameObject {
    pub ob_base: PyVarObject,
    pub f_back: *mut PyFrameObject,
    pub f_code: *mut PyCodeObject,
    pub f_builtins: *mut PyObject,
    pub f_globals: *mut PyObject,
    pub f_locals: *mut PyObject,
    pub f_valuestack: *mut *mut PyObject,

    #[cfg(not(Py_3_10))]
    pub f_stacktop: *mut *mut PyObject,
    pub f_trace: *mut PyObject,
    #[cfg(Py_3_10)]
    pub f_stackdepth: c_int,
    pub f_trace_lines: c_char,
    pub f_trace_opcodes: c_char,

    pub f_gen: *mut PyObject,

    pub f_lasti: c_int,
    pub f_lineno: c_int,
    pub f_iblock: c_int,
    #[cfg(not(Py_3_10))]
    pub f_executing: c_char,
    #[cfg(Py_3_10)]
    pub f_state: PyFrameState,
    pub f_blockstack: [PyTryBlock; crate::CO_MAXBLOCKS],
    pub f_localsplus: [*mut PyObject; 1],
}

#[cfg(any(PyPy, Py_3_11))]
opaque_struct!(PyFrameObject);

// skipped _PyFrame_IsRunnable
// skipped _PyFrame_IsExecuting
// skipped _PyFrameHasCompleted

#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyFrame_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyFrame_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyFrame_Type)) as c_int
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyFrame_New")]
    pub fn PyFrame_New(
        tstate: *mut PyThreadState,
        code: *mut PyCodeObject,
        globals: *mut PyObject,
        locals: *mut PyObject,
    ) -> *mut PyFrameObject;
    // skipped _PyFrame_New_NoTrack

    pub fn PyFrame_BlockSetup(f: *mut PyFrameObject, _type: c_int, handler: c_int, level: c_int);
    #[cfg(not(any(PyPy, Py_3_11)))]
    pub fn PyFrame_BlockPop(f: *mut PyFrameObject) -> *mut PyTryBlock;

    pub fn PyFrame_LocalsToFast(f: *mut PyFrameObject, clear: c_int);
    pub fn PyFrame_FastToLocalsWithError(f: *mut PyFrameObject) -> c_int;
    pub fn PyFrame_FastToLocals(f: *mut PyFrameObject);

    // skipped _PyFrame_DebugMallocStats
    // skipped PyFrame_GetBack

    #[cfg(not(Py_3_9))]
    pub fn PyFrame_ClearFreeList() -> c_int;
}
