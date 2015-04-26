use libc::{c_char, c_int};
use object::*;

#[allow(missing_copy_implementations)]
pub enum PyCodeObject { /* hidden representation */ }

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

pub const CO_FUTURE_DIVISION : c_int = 0x2000;
pub const CO_FUTURE_ABSOLUTE_IMPORT : c_int = 0x4000; /* do absolute imports by default */
pub const CO_FUTURE_WITH_STATEMENT : c_int = 0x8000;
pub const CO_FUTURE_PRINT_FUNCTION : c_int = 0x10000;
pub const CO_FUTURE_UNICODE_LITERALS : c_int = 0x20000;

#[link(name = "python2.7")]
extern "C" {
    pub static mut PyCode_Type: PyTypeObject;
    
    pub fn PyCode_New(arg1: c_int, arg2: c_int,
                      arg3: c_int, arg4: c_int,
                      arg5: *mut PyObject, arg6: *mut PyObject,
                      arg7: *mut PyObject, arg8: *mut PyObject,
                      arg9: *mut PyObject, arg10: *mut PyObject,
                      arg11: *mut PyObject, arg12: *mut PyObject,
                      arg13: c_int, arg14: *mut PyObject)
     -> *mut PyCodeObject;
    pub fn PyCode_NewEmpty(filename: *const c_char,
                           funcname: *const c_char,
                           firstlineno: c_int) -> *mut PyCodeObject;
    pub fn PyCode_Addr2Line(arg1: *mut PyCodeObject, arg2: c_int)
     -> c_int;
    //fn _PyCode_CheckLineNumber(co: *mut PyCodeObject,
    //                               lasti: c_int,
    //                               bounds: *mut PyAddrPair) -> c_int;
    pub fn PyCode_Optimize(code: *mut PyObject, consts: *mut PyObject,
                           names: *mut PyObject, lineno_obj: *mut PyObject)
     -> *mut PyObject;
}

#[inline(always)]
pub unsafe fn PyCode_Check(op : *mut PyObject) -> c_int {
    let u : *mut PyTypeObject = &mut PyCode_Type;
    (Py_TYPE(op) == u) as c_int
}

