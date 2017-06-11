use std::os::raw::c_int;
use ffi2::object::*;
use ffi2::pyport::Py_ssize_t;
use ffi2::code::{PyCodeObject, CO_MAXBLOCKS};
use ffi2::pystate::PyThreadState;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyTryBlock {
    pub b_type : c_int,
    pub b_handler : c_int,
    pub b_level : c_int,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyFrameObject {
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_next: *mut PyObject,
    #[cfg(py_sys_config="Py_TRACE_REFS")]
    pub _ob_prev: *mut PyObject,
    pub ob_refcnt: Py_ssize_t,
    pub ob_type: *mut PyTypeObject,
    pub ob_size: Py_ssize_t,
    pub f_back: *mut PyFrameObject,	/* previous frame, or NULL */
    pub f_code: *mut PyCodeObject,	/* code segment */
    pub f_builtins: *mut PyObject,	/* builtin symbol table (PyDictObject) */
    pub f_globals: *mut PyObject,	/* global symbol table (PyDictObject) */
    pub f_locals: *mut PyObject,		/* local symbol table (any mapping) */
    pub f_valuestack: *mut *mut PyObject,	/* points after the last local */
    /* Next free slot in f_valuestack.  Frame creation sets to f_valuestack.
       Frame evaluation usually NULLs it, but a frame that yields sets it
       to the current stack top. */
    pub f_stacktop: *mut *mut PyObject,
    pub f_trace: *mut PyObject,		/* Trace function */

    pub f_exc_type: *mut PyObject,
    pub f_exc_value: *mut PyObject,
    pub f_exc_traceback: *mut PyObject,

    pub f_tstate: *mut PyThreadState,

    pub f_lasti: c_int,		/* Last instruction if called */
    /* Call PyFrame_GetLineNumber() instead of reading this field
       directly.  As of 2.3 f_lineno is only valid when tracing is
       active (i.e. when f_trace is set).  At other times we use
       PyCode_Addr2Line to calculate the line from the current
      bytecode index. */
    pub f_lineno: c_int,		/* Current line number */
    pub f_iblock: c_int,		/* index in f_blockstack */
    pub f_blockstack: [PyTryBlock; CO_MAXBLOCKS], /* for try and loop blocks */
    pub f_localsplus: [*mut PyObject; 1]	/* locals+stack, dynamically sized */
}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyFrame_Type: PyTypeObject;
}

#[inline]
pub unsafe fn PyFrame_Check(op: *mut PyObject) -> c_int {
     ((*op).ob_type == &mut PyFrame_Type) as c_int
}

//#[inline]
//pub unsafe fn PyFrame_IsRestricted(f: *mut PyFrameObject) -> c_int {
//     ((*f).f_builtins != (*(*(*f).f_tstate).interp).builtins) as c_int
//}

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyFrame_New(tstate: *mut PyThreadState, code: *mut PyCodeObject,
        globals: *mut PyObject, locals: *mut PyObject) -> *mut PyFrameObject;

    pub fn PyFrame_BlockSetup(f: *mut PyFrameObject, _type: c_int, handler: c_int, level: c_int) -> ();
    pub fn PyFrame_BlockPop(f: *mut PyFrameObject) -> *mut PyTryBlock;

    pub fn PyFrame_LocalsToFast(f: *mut PyFrameObject, clear: c_int) -> ();
    pub fn PyFrame_FastToLocals(f: *mut PyFrameObject) -> ();
    
    pub fn PyFrame_ClearFreeList() -> c_int;
    pub fn PyFrame_GetLineNumber(f: *mut PyFrameObject) -> c_int;
}

