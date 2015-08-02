use libc::{c_char, c_int, wchar_t};
use object::*;
use pystate::PyThreadState;

pub const Py_single_input: c_int = 256;
pub const Py_file_input: c_int = 257;
pub const Py_eval_input: c_int = 258;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyCompilerFlags {
    cf_flags : c_int
}

extern "C" {
    pub fn Py_SetProgramName(arg1: *mut wchar_t) -> ();
    pub fn Py_GetProgramName() -> *mut wchar_t;
    pub fn Py_SetPythonHome(arg1: *mut wchar_t) -> ();
    pub fn Py_GetPythonHome() -> *mut wchar_t;
    pub fn Py_Initialize() -> ();
    pub fn Py_InitializeEx(arg1: c_int) -> ();
    pub fn Py_Finalize() -> ();
    pub fn Py_IsInitialized() -> c_int;
    pub fn Py_NewInterpreter() -> *mut PyThreadState;
    pub fn Py_EndInterpreter(arg1: *mut PyThreadState) -> ();
}

pub enum symtable {}
pub enum _node {}

#[inline]
pub unsafe fn PyParser_SimpleParseString(s: *const c_char, b: c_int) -> *mut _node {
    PyParser_SimpleParseStringFlags(s, b, 0)
}

extern "C" {
    pub fn PyParser_SimpleParseStringFlags(arg1: *const c_char,
                                           arg2: c_int,
                                           arg3: c_int)
     -> *mut _node;
    pub fn PyParser_SimpleParseStringFlagsFilename(arg1:
                                                       *const c_char,
                                                   arg2:
                                                       *const c_char,
                                                   arg3: c_int,
                                                   arg4: c_int)
     -> *mut _node;
    //pub fn PyParser_SimpleParseFileFlags(arg1: *mut FILE,
    //                                     arg2: *const c_char,
    //                                     arg3: c_int,
    //                                     arg4: c_int)
    // -> *mut _node;

    pub fn PyRun_StringFlags(code: *const c_char, start: c_int,
                             globals: *mut PyObject, locals: *mut PyObject,
                             flags: *mut PyCompilerFlags) -> *mut PyObject;

    pub fn Py_CompileString(arg1: *const c_char,
                            arg2: *const c_char, arg3: c_int)
     -> *mut PyObject;
    pub fn Py_SymtableString(str: *const c_char,
                             filename: *const c_char,
                             start: c_int) -> *mut symtable;

    pub fn PyErr_Print() -> ();
    pub fn PyErr_PrintEx(arg1: c_int) -> ();
    pub fn PyErr_Display(arg1: *mut PyObject, arg2: *mut PyObject,
                         arg3: *mut PyObject) -> ();
    pub fn Py_AtExit(func: ::std::option::Option<extern "C" fn() -> ()>)
     -> c_int;
    pub fn Py_Exit(arg1: c_int) -> ();
    pub fn Py_Main(argc: c_int, argv: *mut *mut wchar_t)
     -> c_int;
    pub fn Py_GetProgramFullPath() -> *mut wchar_t;
    pub fn Py_GetPrefix() -> *mut wchar_t;
    pub fn Py_GetExecPrefix() -> *mut wchar_t;
    pub fn Py_GetPath() -> *mut wchar_t;
    pub fn Py_SetPath(arg1: *const wchar_t) -> ();
    pub fn Py_GetVersion() -> *const c_char;
    pub fn Py_GetPlatform() -> *const c_char;
    pub fn Py_GetCopyright() -> *const c_char;
    pub fn Py_GetCompiler() -> *const c_char;
    pub fn Py_GetBuildInfo() -> *const c_char;
}

