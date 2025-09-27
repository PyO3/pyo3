use crate::object::*;
#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
use libc::FILE;
#[cfg(any(Py_LIMITED_API, not(Py_3_10), PyPy, GraalPy))]
use std::ffi::c_char;
use std::ffi::c_int;

extern "C" {
    #[cfg(any(all(Py_LIMITED_API, not(PyPy)), GraalPy))]
    pub fn Py_CompileString(string: *const c_char, p: *const c_char, s: c_int) -> *mut PyObject;

    #[cfg_attr(PyPy, link_name = "PyPyErr_Print")]
    pub fn PyErr_Print();
    #[cfg_attr(PyPy, link_name = "PyPyErr_PrintEx")]
    pub fn PyErr_PrintEx(arg1: c_int);
    #[cfg_attr(PyPy, link_name = "PyPyErr_Display")]
    pub fn PyErr_Display(arg1: *mut PyObject, arg2: *mut PyObject, arg3: *mut PyObject);

    #[cfg(Py_3_12)]
    pub fn PyErr_DisplayException(exc: *mut PyObject);
}

#[inline]
#[cfg(PyPy)]
pub unsafe fn Py_CompileString(string: *const c_char, p: *const c_char, s: c_int) -> *mut PyObject {
    // PyPy's implementation of Py_CompileString always forwards to Py_CompileStringFlags; this
    // is only available in the non-limited API and has a real definition for all versions in
    // the cpython/ subdirectory.
    #[cfg(Py_LIMITED_API)]
    extern "C" {
        #[link_name = "PyPy_CompileStringFlags"]
        pub fn Py_CompileStringFlags(
            string: *const c_char,
            p: *const c_char,
            s: c_int,
            f: *mut std::ffi::c_void, // Actually *mut Py_CompilerFlags in the real definition
        ) -> *mut PyObject;
    }
    #[cfg(not(Py_LIMITED_API))]
    use crate::Py_CompileStringFlags;

    Py_CompileStringFlags(string, p, s, std::ptr::null_mut())
}

// skipped PyOS_InputHook

pub const PYOS_STACK_MARGIN: c_int = 2048;

// skipped PyOS_CheckStack under Microsoft C

#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
opaque_struct!(pub _mod);

#[cfg(not(any(PyPy, Py_3_10)))]
opaque_struct!(pub symtable);
#[cfg(not(any(PyPy, Py_3_10)))]
opaque_struct!(pub _node);

#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
#[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
#[inline]
pub unsafe fn PyParser_SimpleParseString(s: *const c_char, b: c_int) -> *mut _node {
    #[allow(deprecated)]
    crate::PyParser_SimpleParseStringFlags(s, b, 0)
}

#[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
#[cfg_attr(Py_3_9, deprecated(note = "Python 3.9"))]
#[inline]
pub unsafe fn PyParser_SimpleParseFile(fp: *mut FILE, s: *const c_char, b: c_int) -> *mut _node {
    #[allow(deprecated)]
    crate::PyParser_SimpleParseFileFlags(fp, s, b, 0)
}

extern "C" {
    #[cfg(not(any(PyPy, Py_3_10)))]
    pub fn Py_SymtableString(
        str: *const c_char,
        filename: *const c_char,
        start: c_int,
    ) -> *mut symtable;
    #[cfg(not(any(PyPy, Py_LIMITED_API, Py_3_10)))]
    pub fn Py_SymtableStringObject(
        str: *const c_char,
        filename: *mut PyObject,
        start: c_int,
    ) -> *mut symtable;
}
