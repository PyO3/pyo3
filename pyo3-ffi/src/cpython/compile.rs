#[cfg(not(any(PyPy, Py_3_10)))]
use crate::object::PyObject;
#[cfg(not(any(PyPy, Py_3_10)))]
use crate::pyarena::*;
#[cfg(not(any(PyPy, Py_3_10)))]
use crate::pythonrun::*;
#[cfg(not(any(PyPy, Py_3_10)))]
use crate::PyCodeObject;
use crate::INT_MAX;
#[cfg(not(any(PyPy, Py_3_10)))]
use std::ffi::c_char;
use std::ffi::c_int;

// skipped PyCF_MASK
// skipped PyCF_MASK_OBSOLETE
// skipped PyCF_SOURCE_IS_UTF8
// skipped PyCF_DONT_IMPLY_DEDENT
// skipped PyCF_ONLY_AST
// skipped PyCF_IGNORE_COOKIE
// skipped PyCF_TYPE_COMMENTS
// skipped PyCF_ALLOW_TOP_LEVEL_AWAIT
// skipped PyCF_OPTIMIZED_AST
// skipped PyCF_COMPILE_MASK

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyCompilerFlags {
    pub cf_flags: c_int,
    #[cfg(Py_3_8)]
    pub cf_feature_version: c_int,
}

// skipped _PyCompilerFlags_INIT

// NB this type technically existed in the header until 3.13, when it was
// moved to the internal CPython headers.
//
// We choose not to expose it in the public API past 3.10, as it is
// not used in the public API past that point.
#[cfg(not(any(PyPy, GraalPy, Py_3_10)))]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyFutureFeatures {
    pub ff_features: c_int,
    pub ff_lineno: c_int,
}

// FIXME: these constants should probably be &CStr, if they are used at all

pub const FUTURE_NESTED_SCOPES: &str = "nested_scopes";
pub const FUTURE_GENERATORS: &str = "generators";
pub const FUTURE_DIVISION: &str = "division";
pub const FUTURE_ABSOLUTE_IMPORT: &str = "absolute_import";
pub const FUTURE_WITH_STATEMENT: &str = "with_statement";
pub const FUTURE_PRINT_FUNCTION: &str = "print_function";
pub const FUTURE_UNICODE_LITERALS: &str = "unicode_literals";
pub const FUTURE_BARRY_AS_BDFL: &str = "barry_as_FLUFL";
pub const FUTURE_GENERATOR_STOP: &str = "generator_stop";
pub const FUTURE_ANNOTATIONS: &str = "annotations";

#[cfg(not(any(PyPy, GraalPy, Py_3_10)))]
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
}

pub const PY_INVALID_STACK_EFFECT: c_int = INT_MAX;

extern "C" {

    pub fn PyCompile_OpcodeStackEffect(opcode: c_int, oparg: c_int) -> c_int;

    #[cfg(Py_3_8)]
    pub fn PyCompile_OpcodeStackEffectWithJump(opcode: c_int, oparg: c_int, jump: c_int) -> c_int;
}
