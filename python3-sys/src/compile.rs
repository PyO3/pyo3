use libc::{c_char, c_int};
use object::PyObject;
use pythonrun::*;
use code::*;
use pyarena::*;

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(not(Py_LIMITED_API))]
pub struct PyFutureFeatures {
    pub ff_features: c_int,
    pub ff_lineno: c_int,
}

#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_NESTED_SCOPES    : &'static str = "nested_scopes";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_GENERATORS       : &'static str = "generators";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_DIVISION         : &'static str = "division";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_ABSOLUTE_IMPORT  : &'static str = "absolute_import";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_WITH_STATEMENT   : &'static str = "with_statement";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_PRINT_FUNCTION   : &'static str = "print_function";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_UNICODE_LITERALS : &'static str = "unicode_literals";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_BARRY_AS_BDFL    : &'static str = "barry_as_FLUFL";
#[cfg(all(not(Py_LIMITED_API), Py_3_5))]
pub const FUTURE_GENERATOR_STOP   : &'static str = "generator_stop";

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PyNode_Compile(arg1: *mut _node,
                          arg2: *const c_char) -> *mut PyCodeObject;
    pub fn PyAST_CompileEx(_mod: *mut _mod,
                           filename: *const c_char,
                           flags: *mut PyCompilerFlags,
                           optimize: c_int, arena: *mut PyArena)
     -> *mut PyCodeObject;
    #[cfg(Py_3_4)]
    pub fn PyAST_CompileObject(_mod: *mut _mod,
                               filename: *mut PyObject,
                               flags: *mut PyCompilerFlags,
                               optimize: c_int, arena: *mut PyArena)
     -> *mut PyCodeObject;
    pub fn PyFuture_FromAST(_mod: *mut _mod,
                            filename: *const c_char)
     -> *mut PyFutureFeatures;
    #[cfg(Py_3_4)]
    pub fn PyFuture_FromASTObject(_mod: *mut _mod,
                                  filename: *mut PyObject)
     -> *mut PyFutureFeatures;
    #[cfg(Py_3_4)]
    pub fn PyCompile_OpcodeStackEffect(opcode: c_int,
                                       oparg: c_int) -> c_int;
}

pub const Py_single_input: c_int = 256;
pub const Py_file_input: c_int = 257;
pub const Py_eval_input: c_int = 258;

