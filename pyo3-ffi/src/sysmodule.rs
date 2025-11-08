use crate::object::PyObject;
use libc::wchar_t;
use std::ffi::{c_char, c_int};

extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPySys_GetObject")]
    pub fn PySys_GetObject(arg1: *const c_char) -> *mut PyObject;
    #[cfg_attr(PyPy, link_name = "PyPySys_SetObject")]
    pub fn PySys_SetObject(arg1: *const c_char, arg2: *mut PyObject) -> c_int;

    #[cfg_attr(
        Py_3_11,
        deprecated(
            note = "Deprecated in Python 3.11, use `PyConfig.argv` and `PyConfig.parse_argv` instead"
        )
    )]
    pub fn PySys_SetArgv(arg1: c_int, arg2: *mut *mut wchar_t);
    #[cfg_attr(
        Py_3_11,
        deprecated(
            note = "Deprecated in Python 3.11, use `PyConfig.argv` and `PyConfig.parse_argv` instead"
        )
    )]
    pub fn PySys_SetArgvEx(arg1: c_int, arg2: *mut *mut wchar_t, arg3: c_int);
    pub fn PySys_SetPath(arg1: *const wchar_t);

    #[cfg_attr(PyPy, link_name = "PyPySys_WriteStdout")]
    pub fn PySys_WriteStdout(format: *const c_char, ...);
    #[cfg_attr(PyPy, link_name = "PyPySys_WriteStderr")]
    pub fn PySys_WriteStderr(format: *const c_char, ...);
    pub fn PySys_FormatStdout(format: *const c_char, ...);
    pub fn PySys_FormatStderr(format: *const c_char, ...);

    #[cfg_attr(
        Py_3_13,
        deprecated(
            note = "Deprecated since Python 3.13. Clear sys.warnoptions and warnings.filters instead."
        )
    )]
    pub fn PySys_ResetWarnOptions();
    #[cfg_attr(Py_3_11, deprecated(note = "Python 3.11"))]
    pub fn PySys_AddWarnOption(arg1: *const wchar_t);
    #[cfg_attr(Py_3_11, deprecated(note = "Python 3.11"))]
    pub fn PySys_AddWarnOptionUnicode(arg1: *mut PyObject);
    #[cfg_attr(Py_3_11, deprecated(note = "Python 3.11"))]
    pub fn PySys_HasWarnOptions() -> c_int;

    pub fn PySys_AddXOption(arg1: *const wchar_t);
    pub fn PySys_GetXOptions() -> *mut PyObject;
}
