use crate::object::*;
#[cfg(not(any(PyPy, GraalPy, Py_LIMITED_API, Py_3_10)))]
use crate::pyarena::PyArena;
use crate::PyCompilerFlags;
#[cfg(not(any(PyPy, GraalPy, Py_3_10)))]
use crate::{_mod, _node};
use libc::FILE;
use std::ffi::{c_char, c_int};

extern "C" {
    pub fn PyRun_SimpleStringFlags(arg1: *const c_char, arg2: *mut PyCompilerFlags) -> c_int;
    pub fn _PyRun_SimpleFileObject(
        fp: *mut FILE,
        filename: *mut PyObject,
        closeit: c_int,
        flags: *mut PyCompilerFlags,
    ) -> c_int;
    pub fn PyRun_AnyFileExFlags(
        fp: *mut FILE,
        filename: *const c_char,
        closeit: c_int,
        flags: *mut PyCompilerFlags,
    ) -> c_int;
    pub fn _PyRun_AnyFileObject(
        fp: *mut FILE,
        filename: *mut PyObject,
        closeit: c_int,
        flags: *mut PyCompilerFlags,
    ) -> c_int;
    pub fn PyRun_SimpleFileExFlags(
        fp: *mut FILE,
        filename: *const c_char,
        closeit: c_int,
        flags: *mut PyCompilerFlags,
    ) -> c_int;
    pub fn PyRun_InteractiveOneFlags(
        fp: *mut FILE,
        filename: *const c_char,
        flags: *mut PyCompilerFlags,
    ) -> c_int;
    pub fn PyRun_InteractiveOneObject(
        fp: *mut FILE,
        filename: *mut PyObject,
        flags: *mut PyCompilerFlags,
    ) -> c_int;
    pub fn PyRun_InteractiveLoopFlags(
        fp: *mut FILE,
        filename: *const c_char,
        flags: *mut PyCompilerFlags,
    ) -> c_int;
    pub fn _PyRun_InteractiveLoopObject(
        fp: *mut FILE,
        filename: *mut PyObject,
        flags: *mut PyCompilerFlags,
    ) -> c_int;

    #[cfg(not(any(PyPy, GraalPy, Py_3_10)))]
    pub fn PyParser_ASTFromString(
        s: *const c_char,
        filename: *const c_char,
        start: c_int,
        flags: *mut PyCompilerFlags,
        arena: *mut PyArena,
    ) -> *mut _mod;
    #[cfg(not(any(PyPy, GraalPy, Py_3_10)))]
    pub fn PyParser_ASTFromStringObject(
        s: *const c_char,
        filename: *mut PyObject,
        start: c_int,
        flags: *mut PyCompilerFlags,
        arena: *mut PyArena,
    ) -> *mut _mod;
    #[cfg(not(any(PyPy, GraalPy, Py_3_10)))]
    pub fn PyParser_ASTFromFile(
        fp: *mut FILE,
        filename: *const c_char,
        enc: *const c_char,
        start: c_int,
        ps1: *const c_char,
        ps2: *const c_char,
        flags: *mut PyCompilerFlags,
        errcode: *mut c_int,
        arena: *mut PyArena,
    ) -> *mut _mod;
    #[cfg(not(any(PyPy, GraalPy, Py_3_10)))]
    pub fn PyParser_ASTFromFileObject(
        fp: *mut FILE,
        filename: *mut PyObject,
        enc: *const c_char,
        start: c_int,
        ps1: *const c_char,
        ps2: *const c_char,
        flags: *mut PyCompilerFlags,
        errcode: *mut c_int,
        arena: *mut PyArena,
    ) -> *mut _mod;
}

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyRun_StringFlags")]
    pub fn PyRun_StringFlags(
        arg1: *const c_char,
        arg2: c_int,
        arg3: *mut PyObject,
        arg4: *mut PyObject,
        arg5: *mut PyCompilerFlags,
    ) -> *mut PyObject;
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyRun_FileExFlags(
        fp: *mut FILE,
        filename: *const c_char,
        start: c_int,
        globals: *mut PyObject,
        locals: *mut PyObject,
        closeit: c_int,
        flags: *mut PyCompilerFlags,
    ) -> *mut PyObject;

    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn Py_CompileStringExFlags(
        str: *const c_char,
        filename: *const c_char,
        start: c_int,
        flags: *mut PyCompilerFlags,
        optimize: c_int,
    ) -> *mut PyObject;
    #[cfg(not(Py_LIMITED_API))]
    pub fn Py_CompileStringObject(
        str: *const c_char,
        filename: *mut PyObject,
        start: c_int,
        flags: *mut PyCompilerFlags,
        optimize: c_int,
    ) -> *mut PyObject;
}

#[inline]
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn Py_CompileString(string: *const c_char, p: *const c_char, s: c_int) -> *mut PyObject {
    Py_CompileStringExFlags(string, p, s, std::ptr::null_mut(), -1)
}

#[inline]
#[cfg(not(any(PyPy, GraalPy)))]
pub unsafe fn Py_CompileStringFlags(
    string: *const c_char,
    p: *const c_char,
    s: c_int,
    f: *mut PyCompilerFlags,
) -> *mut PyObject {
    Py_CompileStringExFlags(string, p, s, f, -1)
}

// skipped _Py_SourceAsString

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPyRun_String")]
    pub fn PyRun_String(
        string: *const c_char,
        s: c_int,
        g: *mut PyObject,
        l: *mut PyObject,
    ) -> *mut PyObject;
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyRun_AnyFile(fp: *mut FILE, name: *const c_char) -> c_int;
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyRun_AnyFileEx(fp: *mut FILE, name: *const c_char, closeit: c_int) -> c_int;
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyRun_AnyFileFlags(
        arg1: *mut FILE,
        arg2: *const c_char,
        arg3: *mut PyCompilerFlags,
    ) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyRun_SimpleString")]
    pub fn PyRun_SimpleString(s: *const c_char) -> c_int;
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyRun_SimpleFile(f: *mut FILE, p: *const c_char) -> c_int;
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyRun_SimpleFileEx(f: *mut FILE, p: *const c_char, c: c_int) -> c_int;
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyRun_InteractiveOne(f: *mut FILE, p: *const c_char) -> c_int;
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyRun_InteractiveLoop(f: *mut FILE, p: *const c_char) -> c_int;
    #[cfg_attr(PyPy, link_name = "PyPyRun_File")]
    pub fn PyRun_File(
        fp: *mut FILE,
        p: *const c_char,
        s: c_int,
        g: *mut PyObject,
        l: *mut PyObject,
    ) -> *mut PyObject;
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyRun_FileEx(
        fp: *mut FILE,
        p: *const c_char,
        s: c_int,
        g: *mut PyObject,
        l: *mut PyObject,
        c: c_int,
    ) -> *mut PyObject;
    #[cfg(not(any(PyPy, GraalPy)))]
    pub fn PyRun_FileFlags(
        fp: *mut FILE,
        p: *const c_char,
        s: c_int,
        g: *mut PyObject,
        l: *mut PyObject,
        flags: *mut PyCompilerFlags,
    ) -> *mut PyObject;
}

// skipped macro PyRun_String
// skipped macro PyRun_AnyFile
// skipped macro PyRun_AnyFileEx
// skipped macro PyRun_AnyFileFlags

extern "C" {
    #[cfg(not(any(PyPy, GraalPy, Py_3_10)))]
    #[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
    pub fn PyParser_SimpleParseStringFlags(
        arg1: *const c_char,
        arg2: c_int,
        arg3: c_int,
    ) -> *mut _node;
    #[cfg(not(any(PyPy, GraalPy, Py_3_10)))]
    #[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
    pub fn PyParser_SimpleParseStringFlagsFilename(
        arg1: *const c_char,
        arg2: *const c_char,
        arg3: c_int,
        arg4: c_int,
    ) -> *mut _node;
    #[cfg(not(any(PyPy, GraalPy, Py_3_10)))]
    #[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
    pub fn PyParser_SimpleParseFileFlags(
        arg1: *mut FILE,
        arg2: *const c_char,
        arg3: c_int,
        arg4: c_int,
    ) -> *mut _node;

    #[cfg(PyPy)]
    #[cfg_attr(PyPy, link_name = "PyPy_CompileStringFlags")]
    pub fn Py_CompileStringFlags(
        string: *const c_char,
        p: *const c_char,
        s: c_int,
        f: *mut PyCompilerFlags,
    ) -> *mut PyObject;
}
