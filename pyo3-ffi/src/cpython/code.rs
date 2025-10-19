use crate::object::*;
use crate::pyport::Py_ssize_t;

#[cfg(not(GraalPy))]
use std::ffi::c_char;
use std::ffi::{c_int, c_void};
#[cfg(not(PyPy))]
use std::ptr::addr_of_mut;

// skipped private _PY_MONITORING_LOCAL_EVENTS
// skipped private _PY_MONITORING_UNGROUPED_EVENTS
// skipped private _PY_MONITORING_EVENTS

// skipped private _PyLocalMonitors
// skipped private _Py_GlobalMonitors

// skipped private _Py_CODEUNIT

// skipped private _Py_OPCODE
// skipped private _Py_OPARG

// skipped private _py_make_codeunit

// skipped private _py_set_opcode

// skipped private _Py_MAKE_CODEUNIT
// skipped private _Py_SET_OPCODE

// skipped private _PyCoCached
// skipped private _PyCoLineInstrumentationData
// skipped private _PyCoMontoringData

// skipped private _PyExecutorArray

opaque_struct!(
    #[doc = "A Python code object.\n"]
    #[doc = "\n"]
    #[doc = "`pyo3-ffi` does not expose the contents of this struct, as it has no stability guarantees."]
    pub PyCodeObject
);

/* Masks for co_flags */
pub const CO_OPTIMIZED: c_int = 0x0001;
pub const CO_NEWLOCALS: c_int = 0x0002;
pub const CO_VARARGS: c_int = 0x0004;
pub const CO_VARKEYWORDS: c_int = 0x0008;
pub const CO_NESTED: c_int = 0x0010;
pub const CO_GENERATOR: c_int = 0x0020;
/* The CO_NOFREE flag is set if there are no free or cell variables.
   This information is redundant, but it allows a single flag test
   to determine whether there is any extra work to be done when the
   call frame it setup.
*/
pub const CO_NOFREE: c_int = 0x0040;
/* The CO_COROUTINE flag is set for coroutine functions (defined with
``async def`` keywords) */
pub const CO_COROUTINE: c_int = 0x0080;
pub const CO_ITERABLE_COROUTINE: c_int = 0x0100;
pub const CO_ASYNC_GENERATOR: c_int = 0x0200;

pub const CO_FUTURE_DIVISION: c_int = 0x2000;
pub const CO_FUTURE_ABSOLUTE_IMPORT: c_int = 0x4000; /* do absolute imports by default */
pub const CO_FUTURE_WITH_STATEMENT: c_int = 0x8000;
pub const CO_FUTURE_PRINT_FUNCTION: c_int = 0x1_0000;
pub const CO_FUTURE_UNICODE_LITERALS: c_int = 0x2_0000;

pub const CO_FUTURE_BARRY_AS_BDFL: c_int = 0x4_0000;
pub const CO_FUTURE_GENERATOR_STOP: c_int = 0x8_0000;
// skipped CO_FUTURE_ANNOTATIONS
// skipped CO_CELL_NOT_AN_ARG

pub const CO_MAXBLOCKS: usize = 20;

#[cfg(not(PyPy))]
#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static mut PyCode_Type: PyTypeObject;
}

#[inline]
#[cfg(not(PyPy))]
pub unsafe fn PyCode_Check(op: *mut PyObject) -> c_int {
    (Py_TYPE(op) == addr_of_mut!(PyCode_Type)) as c_int
}

extern "C" {
    #[cfg(PyPy)]
    #[link_name = "PyPyCode_Check"]
    pub fn PyCode_Check(op: *mut PyObject) -> c_int;
}

// skipped PyCode_GetNumFree (requires knowledge of code object layout)

extern "C" {
    #[cfg(not(GraalPy))]
    #[cfg_attr(PyPy, link_name = "PyPyCode_New")]
    pub fn PyCode_New(
        argcount: c_int,
        kwonlyargcount: c_int,
        nlocals: c_int,
        stacksize: c_int,
        flags: c_int,
        code: *mut PyObject,
        consts: *mut PyObject,
        names: *mut PyObject,
        varnames: *mut PyObject,
        freevars: *mut PyObject,
        cellvars: *mut PyObject,
        filename: *mut PyObject,
        name: *mut PyObject,
        firstlineno: c_int,
        lnotab: *mut PyObject,
    ) -> *mut PyCodeObject;
    #[cfg(not(GraalPy))]
    #[cfg(Py_3_8)]
    pub fn PyCode_NewWithPosOnlyArgs(
        argcount: c_int,
        posonlyargcount: c_int,
        kwonlyargcount: c_int,
        nlocals: c_int,
        stacksize: c_int,
        flags: c_int,
        code: *mut PyObject,
        consts: *mut PyObject,
        names: *mut PyObject,
        varnames: *mut PyObject,
        freevars: *mut PyObject,
        cellvars: *mut PyObject,
        filename: *mut PyObject,
        name: *mut PyObject,
        firstlineno: c_int,
        lnotab: *mut PyObject,
    ) -> *mut PyCodeObject;
    #[cfg(not(GraalPy))]
    #[cfg_attr(PyPy, link_name = "PyPyCode_NewEmpty")]
    pub fn PyCode_NewEmpty(
        filename: *const c_char,
        funcname: *const c_char,
        firstlineno: c_int,
    ) -> *mut PyCodeObject;
    #[cfg(not(GraalPy))]
    pub fn PyCode_Addr2Line(arg1: *mut PyCodeObject, arg2: c_int) -> c_int;
    // skipped PyCodeAddressRange "for internal use only"
    // skipped _PyCode_CheckLineNumber
    // skipped _PyCode_ConstantKey
    pub fn PyCode_Optimize(
        code: *mut PyObject,
        consts: *mut PyObject,
        names: *mut PyObject,
        lnotab: *mut PyObject,
    ) -> *mut PyObject;
    pub fn _PyCode_GetExtra(
        code: *mut PyObject,
        index: Py_ssize_t,
        extra: *const *mut c_void,
    ) -> c_int;
    pub fn _PyCode_SetExtra(code: *mut PyObject, index: Py_ssize_t, extra: *mut c_void) -> c_int;
}
