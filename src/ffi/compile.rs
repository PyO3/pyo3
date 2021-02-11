use crate::ffi::object::PyObject;
use crate::ffi::pyarena::*;
use crate::ffi::pythonrun::*;
use crate::ffi::PyCodeObject;
use std::os::raw::{c_char, c_int};

// skipped non-limited PyCF_MASK
// skipped non-limited PyCF_MASK_OBSOLETE
// skipped non-limited PyCF_SOURCE_IS_UTF8
// skipped non-limited PyCF_DONT_IMPLY_DEDENT
// skipped non-limited PyCF_ONLY_AST
// skipped non-limited PyCF_IGNORE_COOKIE
// skipped non-limited PyCF_TYPE_COMMENTS
// skipped non-limited PyCF_ALLOW_TOP_LEVEL_AWAIT
// skipped non-limited PyCF_COMPILE_MASK

// skipped non-limited PyCompilerFlags
// skipped non-limited _PyCompilerFlags_INIT

#[repr(C)]
#[derive(Copy, Clone)]
#[cfg(not(Py_LIMITED_API))]
pub struct PyFutureFeatures {
    pub ff_features: c_int,
    pub ff_lineno: c_int,
}

#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_NESTED_SCOPES: &str = "nested_scopes";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_GENERATORS: &str = "generators";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_DIVISION: &str = "division";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_ABSOLUTE_IMPORT: &str = "absolute_import";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_WITH_STATEMENT: &str = "with_statement";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_PRINT_FUNCTION: &str = "print_function";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_UNICODE_LITERALS: &str = "unicode_literals";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_BARRY_AS_BDFL: &str = "barry_as_FLUFL";
#[cfg(not(Py_LIMITED_API))]
pub const FUTURE_GENERATOR_STOP: &str = "generator_stop";
// skipped non-limited FUTURE_ANNOTATIONS

#[cfg(not(Py_LIMITED_API))]
extern "C" {
    pub fn PyNode_Compile(arg1: *mut _node, arg2: *const c_char) -> *mut PyCodeObject;

    pub fn PyAST_CompileEx(
        _mod: *mut _mod,
        filename: *const c_char,
        flags: *mut PyCompilerFlags,
        optimize: c_int,
        arena: *mut PyArena,
    ) -> *mut PyCodeObject;

    pub fn PyAST_CompileObject(
        _mod: *mut _mod,
        filename: *mut PyObject,
        flags: *mut PyCompilerFlags,
        optimize: c_int,
        arena: *mut PyArena,
    ) -> *mut PyCodeObject;

    pub fn PyFuture_FromAST(_mod: *mut _mod, filename: *const c_char) -> *mut PyFutureFeatures;

    pub fn PyFuture_FromASTObject(
        _mod: *mut _mod,
        filename: *mut PyObject,
    ) -> *mut PyFutureFeatures;

    // skipped non-limited _Py_Mangle
    // skipped non-limited PY_INVALID_STACK_EFFECT

    pub fn PyCompile_OpcodeStackEffect(opcode: c_int, oparg: c_int) -> c_int;

    #[cfg(Py_3_8)]
    pub fn PyCompile_OpcodeStackEffectWithJump(opcode: c_int, oparg: c_int, jump: c_int) -> c_int;

    // skipped non-limited _PyASTOptimizeState
    // skipped non-limited _PyAST_Optimize
}

pub const Py_single_input: c_int = 256;
pub const Py_file_input: c_int = 257;
pub const Py_eval_input: c_int = 258;
#[cfg(Py_3_8)]
pub const Py_func_type_input: c_int = 345;
// skipped Py_fstring_input
