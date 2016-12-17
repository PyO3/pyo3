use libc::{c_char, c_int, FILE};
use object::*;
use code::*;
use pystate::PyThreadState;
use pyarena::PyArena;

pub const PyCF_MASK : c_int = (CO_FUTURE_DIVISION | CO_FUTURE_ABSOLUTE_IMPORT |
                   CO_FUTURE_WITH_STATEMENT | CO_FUTURE_PRINT_FUNCTION |
                   CO_FUTURE_UNICODE_LITERALS);
pub const PyCF_MASK_OBSOLETE : c_int = (CO_NESTED);
pub const PyCF_SOURCE_IS_UTF8 : c_int = 0x0100;
pub const PyCF_DONT_IMPLY_DEDENT : c_int = 0x0200;
pub const PyCF_ONLY_AST : c_int = 0x0400;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct PyCompilerFlags {
    cf_flags : c_int
}

#[allow(missing_copy_implementations)]
pub enum Struct__mod { }
#[allow(missing_copy_implementations)]
pub enum Struct__node { }
#[allow(missing_copy_implementations)]
pub enum Struct_symtable { }

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn Py_SetProgramName(arg1: *mut c_char);
    pub fn Py_GetProgramName() -> *mut c_char;
    pub fn Py_SetPythonHome(arg1: *mut c_char);
    pub fn Py_GetPythonHome() -> *mut c_char;
    pub fn Py_Initialize();
    pub fn Py_InitializeEx(arg1: c_int);
    pub fn Py_Finalize();
    pub fn Py_IsInitialized() -> c_int;
    pub fn Py_NewInterpreter() -> *mut PyThreadState;
    pub fn Py_EndInterpreter(arg1: *mut PyThreadState);
    pub fn PyRun_AnyFileFlags(arg1: *mut FILE, arg2: *const c_char,
                              arg3: *mut PyCompilerFlags) -> c_int;
    pub fn PyRun_AnyFileExFlags(arg1: *mut FILE, arg2: *const c_char,
                                arg3: c_int,
                                arg4: *mut PyCompilerFlags) -> c_int;
    pub fn PyRun_SimpleStringFlags(arg1: *const c_char,
                                   arg2: *mut PyCompilerFlags)
     -> c_int;
    pub fn PyRun_SimpleFileExFlags(arg1: *mut FILE,
                                   arg2: *const c_char,
                                   arg3: c_int,
                                   arg4: *mut PyCompilerFlags)
     -> c_int;
    pub fn PyRun_InteractiveOneFlags(arg1: *mut FILE,
                                     arg2: *const c_char,
                                     arg3: *mut PyCompilerFlags)
     -> c_int;
    pub fn PyRun_InteractiveLoopFlags(arg1: *mut FILE,
                                      arg2: *const c_char,
                                      arg3: *mut PyCompilerFlags)
     -> c_int;
    pub fn PyParser_ASTFromString(arg1: *const c_char,
                                  arg2: *const c_char,
                                  arg3: c_int,
                                  flags: *mut PyCompilerFlags,
                                  arg4: *mut PyArena) -> *mut Struct__mod;
    pub fn PyParser_ASTFromFile(arg1: *mut FILE, arg2: *const c_char,
                                arg3: c_int,
                                arg4: *mut c_char,
                                arg5: *mut c_char,
                                arg6: *mut PyCompilerFlags,
                                arg7: *mut c_int, arg8: *mut PyArena)
     -> *mut Struct__mod;
    pub fn PyParser_SimpleParseStringFlags(arg1: *const c_char,
                                           arg2: c_int,
                                           arg3: c_int)
     -> *mut Struct__node;
    pub fn PyParser_SimpleParseFileFlags(arg1: *mut FILE,
                                         arg2: *const c_char,
                                         arg3: c_int,
                                         arg4: c_int)
     -> *mut Struct__node;
    pub fn PyRun_StringFlags(arg1: *const c_char, arg2: c_int,
                             arg3: *mut PyObject, arg4: *mut PyObject,
                             arg5: *mut PyCompilerFlags) -> *mut PyObject;
    pub fn PyRun_FileExFlags(arg1: *mut FILE, arg2: *const c_char,
                             arg3: c_int, arg4: *mut PyObject,
                             arg5: *mut PyObject, arg6: c_int,
                             arg7: *mut PyCompilerFlags) -> *mut PyObject;
    pub fn Py_CompileStringFlags(arg1: *const c_char,
                                 arg2: *const c_char,
                                 arg3: c_int,
                                 arg4: *mut PyCompilerFlags) -> *mut PyObject;
    pub fn Py_SymtableString(arg1: *const c_char,
                             arg2: *const c_char, arg3: c_int)
     -> *mut Struct_symtable;
    pub fn PyErr_Print();
    pub fn PyErr_PrintEx(arg1: c_int);
    pub fn PyErr_Display(arg1: *mut PyObject, arg2: *mut PyObject,
                         arg3: *mut PyObject);
    pub fn Py_AtExit(func: Option<unsafe extern "C" fn()>)
     -> c_int;
    pub fn Py_Exit(arg1: c_int);
    pub fn Py_FdIsInteractive(arg1: *mut FILE, arg2: *const c_char)
     -> c_int;
    pub fn Py_Main(argc: c_int, argv: *mut *mut c_char)
     -> c_int;
    pub fn Py_GetProgramFullPath() -> *mut c_char;
    pub fn Py_GetPrefix() -> *mut c_char;
    pub fn Py_GetExecPrefix() -> *mut c_char;
    pub fn Py_GetPath() -> *mut c_char;
    pub fn Py_GetVersion() -> *const c_char;
    pub fn Py_GetPlatform() -> *const c_char;
    pub fn Py_GetCopyright() -> *const c_char;
    pub fn Py_GetCompiler() -> *const c_char;
    pub fn Py_GetBuildInfo() -> *const c_char;
    fn _Py_svnversion() -> *const c_char;
    pub fn Py_SubversionRevision() -> *const c_char;
    pub fn Py_SubversionShortBranch() -> *const c_char;
    fn _Py_hgidentifier() -> *const c_char;
    fn _Py_hgversion() -> *const c_char;
}

