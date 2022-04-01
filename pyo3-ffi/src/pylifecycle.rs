use crate::pystate::PyThreadState;

use libc::wchar_t;
use std::os::raw::{c_char, c_int};

extern "C" {
    pub fn Py_Initialize();
    pub fn Py_InitializeEx(arg1: c_int);
    pub fn Py_Finalize();
    pub fn Py_FinalizeEx() -> c_int;

    #[cfg_attr(PyPy, link_name = "PyPy_IsInitialized")]
    pub fn Py_IsInitialized() -> c_int;

    pub fn Py_NewInterpreter() -> *mut PyThreadState;
    pub fn Py_EndInterpreter(arg1: *mut PyThreadState);

    #[cfg_attr(PyPy, link_name = "PyPy_AtExit")]
    pub fn Py_AtExit(func: Option<extern "C" fn()>) -> c_int;

    pub fn Py_Exit(arg1: c_int);

    pub fn Py_Main(argc: c_int, argv: *mut *mut wchar_t) -> c_int;
    pub fn Py_BytesMain(argc: c_int, argv: *mut *mut c_char) -> c_int;

    pub fn Py_SetProgramName(arg1: *const wchar_t);
    #[cfg_attr(PyPy, link_name = "PyPy_GetProgramName")]
    pub fn Py_GetProgramName() -> *mut wchar_t;

    pub fn Py_SetPythonHome(arg1: *const wchar_t);
    pub fn Py_GetPythonHome() -> *mut wchar_t;

    pub fn Py_GetProgramFullPath() -> *mut wchar_t;

    pub fn Py_GetPrefix() -> *mut wchar_t;
    pub fn Py_GetExecPrefix() -> *mut wchar_t;
    pub fn Py_GetPath() -> *mut wchar_t;
    pub fn Py_SetPath(arg1: *const wchar_t);

    // skipped _Py_CheckPython3

    #[cfg_attr(PyPy, link_name = "PyPy_GetVersion")]
    pub fn Py_GetVersion() -> *const c_char;
    pub fn Py_GetPlatform() -> *const c_char;
    pub fn Py_GetCopyright() -> *const c_char;
    pub fn Py_GetCompiler() -> *const c_char;
    pub fn Py_GetBuildInfo() -> *const c_char;
}

type PyOS_sighandler_t = unsafe extern "C" fn(arg1: c_int);

extern "C" {
    pub fn PyOS_getsig(arg1: c_int) -> PyOS_sighandler_t;
    pub fn PyOS_setsig(arg1: c_int, arg2: PyOS_sighandler_t) -> PyOS_sighandler_t;
}
