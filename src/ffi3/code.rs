use std::os::raw::{c_void, c_uchar, c_char, c_int};
use ffi3::pyport::Py_ssize_t;
use ffi3::object::*;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyCodeObject {
    pub ob_base: PyObject,
    pub co_argcount: c_int,
    pub co_kwonlyargcount: c_int,
    pub co_nlocals: c_int,
    pub co_stacksize: c_int,
    pub co_flags: c_int,
    #[cfg(Py_3_6)]
    pub co_firstlineno: c_int,
    pub co_code: *mut PyObject,
    pub co_consts: *mut PyObject,
    pub co_names: *mut PyObject,
    pub co_varnames: *mut PyObject,
    pub co_freevars: *mut PyObject,
    pub co_cellvars: *mut PyObject,
    pub co_cell2arg: *mut c_uchar,
    pub co_filename: *mut PyObject,
    pub co_name: *mut PyObject,
    #[cfg(not(Py_3_6))]
    pub co_firstlineno: c_int,
    pub co_lnotab: *mut PyObject,
    pub co_zombieframe: *mut c_void,
    pub co_weakreflist: *mut PyObject,
    #[cfg(Py_3_6)]
    pub co_extra: *mut c_void,
}

impl Default for PyCodeObject {
    #[inline] fn default() -> Self { unsafe { ::std::mem::zeroed() } }
}

/* Masks for co_flags */
pub const CO_OPTIMIZED : c_int = 0x0001;
pub const CO_NEWLOCALS : c_int = 0x0002;
pub const CO_VARARGS : c_int = 0x0004;
pub const CO_VARKEYWORDS : c_int = 0x0008;
pub const CO_NESTED : c_int = 0x0010;
pub const CO_GENERATOR : c_int = 0x0020;
/* The CO_NOFREE flag is set if there are no free or cell variables.
   This information is redundant, but it allows a single flag test
   to determine whether there is any extra work to be done when the
   call frame it setup.
*/
pub const CO_NOFREE : c_int = 0x0040;
/* The CO_COROUTINE flag is set for coroutine functions (defined with
   ``async def`` keywords) */
pub const CO_COROUTINE : c_int = 0x0080;
pub const CO_ITERABLE_COROUTINE : c_int = 0x0100;
pub const CO_ASYNC_GENERATOR : c_int = 0x0200;

pub const CO_FUTURE_DIVISION : c_int = 0x2000;
pub const CO_FUTURE_ABSOLUTE_IMPORT : c_int = 0x4000; /* do absolute imports by default */
pub const CO_FUTURE_WITH_STATEMENT : c_int = 0x8000;
pub const CO_FUTURE_PRINT_FUNCTION : c_int = 0x1_0000;
pub const CO_FUTURE_UNICODE_LITERALS : c_int = 0x2_0000;
pub const CO_FUTURE_BARRY_AS_BDFL : c_int = 0x4_0000;
pub const CO_FUTURE_GENERATOR_STOP : c_int = 0x8_0000;

pub const CO_MAXBLOCKS: usize = 20;

#[cfg(Py_3_6)]
pub type FreeFunc = extern "C" fn(*mut c_void) -> c_void;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub static mut PyCode_Type: PyTypeObject;
    #[cfg(Py_3_6)]
    pub fn _PyCode_GetExtra(code: *mut PyObject, index: Py_ssize_t,
                            extra: *const *mut c_void) -> c_int;
    #[cfg(Py_3_6)]
    pub fn _PyCode_SetExtra(code: *mut PyObject, index: Py_ssize_t,
                            extra: *mut c_void) -> c_int;

    pub fn PyCode_New(arg1: c_int, arg2: c_int,
                      arg3: c_int, arg4: c_int,
                      arg5: c_int, arg6: *mut PyObject,
                      arg7: *mut PyObject, arg8: *mut PyObject,
                      arg9: *mut PyObject, arg10: *mut PyObject,
                      arg11: *mut PyObject, arg12: *mut PyObject,
                      arg13: *mut PyObject, arg14: c_int,
                      arg15: *mut PyObject) -> *mut PyCodeObject;
    pub fn PyCode_NewEmpty(filename: *const c_char,
                           funcname: *const c_char,
                           firstlineno: c_int) -> *mut PyCodeObject;
    pub fn PyCode_Addr2Line(arg1: *mut PyCodeObject, arg2: c_int)
     -> c_int;
    pub fn PyCode_Optimize(code: *mut PyObject, consts: *mut PyObject,
                           names: *mut PyObject, lnotab: *mut PyObject)
     -> *mut PyObject;
}

#[inline]
pub unsafe fn PyCode_Check(op : *mut PyObject) -> c_int {
    (Py_TYPE(op) == &mut PyCode_Type) as c_int
}

#[inline]
pub unsafe fn PyCode_GetNumFree(op : *mut PyCodeObject) -> Py_ssize_t {
    ::ffi3::tupleobject::PyTuple_GET_SIZE((*op).co_freevars)
}
