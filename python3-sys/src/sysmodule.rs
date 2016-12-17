use libc::{c_char, c_int, wchar_t};
use object::PyObject;

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    pub fn PySys_GetObject(arg1: *const c_char) -> *mut PyObject;
    pub fn PySys_SetObject(arg1: *const c_char, arg2: *mut PyObject)
     -> c_int;
    pub fn PySys_SetArgv(arg1: c_int, arg2: *mut *mut wchar_t) -> ();
    pub fn PySys_SetArgvEx(arg1: c_int, arg2: *mut *mut wchar_t,
                           arg3: c_int) -> ();
    pub fn PySys_SetPath(arg1: *const wchar_t) -> ();
    pub fn PySys_WriteStdout(format: *const c_char, ...) -> ();
    pub fn PySys_WriteStderr(format: *const c_char, ...) -> ();
    pub fn PySys_FormatStdout(format: *const c_char, ...) -> ();
    pub fn PySys_FormatStderr(format: *const c_char, ...) -> ();
    pub fn PySys_ResetWarnOptions() -> ();
    pub fn PySys_AddWarnOption(arg1: *const wchar_t) -> ();
    pub fn PySys_AddWarnOptionUnicode(arg1: *mut PyObject) -> ();
    pub fn PySys_HasWarnOptions() -> c_int;
    pub fn PySys_AddXOption(arg1: *const wchar_t) -> ();
    pub fn PySys_GetXOptions() -> *mut PyObject;
}

