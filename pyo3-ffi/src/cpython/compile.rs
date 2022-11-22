#[cfg(not(any(PyPy, Py_3_10)))]
use crate::object::PyObject;
#[cfg(not(any(PyPy, Py_3_10)))]
use crate::pyarena::*;
#[cfg(not(any(PyPy, Py_3_10)))]
use crate::pythonrun::*;
#[cfg(not(any(PyPy, Py_3_10)))]
use crate::PyCodeObject;
#[cfg(not(any(PyPy, Py_3_10)))]
use std::os::raw::c_char;
use std::os::raw::c_int;

// skipped non-limited PyCF_MASK
// skipped non-limited PyCF_MASK_OBSOLETE
// skipped non-limited PyCF_SOURCE_IS_UTF8
// skipped non-limited PyCF_DONT_IMPLY_DEDENT
// skipped non-limited PyCF_ONLY_AST
// skipped non-limited PyCF_IGNORE_COOKIE
// skipped non-limited PyCF_TYPE_COMMENTS
// skipped non-limited PyCF_ALLOW_TOP_LEVEL_AWAIT
// skipped non-limited PyCF_COMPILE_MASK

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyCompilerFlags {
    pub cf_flags: c_int,
    #[cfg(Py_3_8)]
    pub cf_feature_version: c_int,
}

// skipped non-limited _PyCompilerFlags_INIT

#[cfg(all(Py_3_12, not(PyPy)))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct _PyCompilerSrcLocation {
    pub lineno: c_int,
    pub end_lineno: c_int,
    pub col_offset: c_int,
    pub end_col_offset: c_int,
}

// skipped SRC_LOCATION_FROM_AST

#[cfg(not(PyPy))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyFutureFeatures {
    pub ff_features: c_int,
    #[cfg(not(Py_3_12))]
    pub ff_lineno: c_int,
    #[cfg(Py_3_12)]
    pub ff_location: _PyCompilerSrcLocation,
}

pub const FUTURE_NESTED_SCOPES: &str = "nested_scopes";
pub const FUTURE_GENERATORS: &str = "generators";
pub const FUTURE_DIVISION: &str = "division";
pub const FUTURE_ABSOLUTE_IMPORT: &str = "absolute_import";
pub const FUTURE_WITH_STATEMENT: &str = "with_statement";
pub const FUTURE_PRINT_FUNCTION: &str = "print_function";
pub const FUTURE_UNICODE_LITERALS: &str = "unicode_literals";
pub const FUTURE_BARRY_AS_BDFL: &str = "barry_as_FLUFL";
pub const FUTURE_GENERATOR_STOP: &str = "generator_stop";
// skipped non-limited FUTURE_ANNOTATIONS

extern "C" {
    #[cfg(not(any(PyPy, Py_3_10)))]
    pub fn PyNode_Compile(arg1: *mut _node, arg2: *const c_char) -> *mut PyCodeObject;

    #[cfg(not(any(PyPy, Py_3_10)))]
    pub fn PyAST_CompileEx(
        _mod: *mut _mod,
        filename: *const c_char,
        flags: *mut PyCompilerFlags,
        optimize: c_int,
        arena: *mut PyArena,
    ) -> *mut PyCodeObject;

    #[cfg(not(any(PyPy, Py_3_10)))]
    pub fn PyAST_CompileObject(
        _mod: *mut _mod,
        filename: *mut PyObject,
        flags: *mut PyCompilerFlags,
        optimize: c_int,
        arena: *mut PyArena,
    ) -> *mut PyCodeObject;

    #[cfg(not(any(PyPy, Py_3_10)))]
    pub fn PyFuture_FromAST(_mod: *mut _mod, filename: *const c_char) -> *mut PyFutureFeatures;

    #[cfg(not(any(PyPy, Py_3_10)))]
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
