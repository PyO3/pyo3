use std::ffi::{c_char, c_int};

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_DebugFlag")]
    pub static mut Py_DebugFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_VerboseFlag")]
    pub static mut Py_VerboseFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    pub static mut Py_QuietFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_InteractiveFlag")]
    pub static mut Py_InteractiveFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_InspectFlag")]
    pub static mut Py_InspectFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_OptimizeFlag")]
    pub static mut Py_OptimizeFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_NoSiteFlag")]
    pub static mut Py_NoSiteFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_BytesWarningFlag")]
    pub static mut Py_BytesWarningFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_UseClassExceptionsFlag")]
    pub static mut Py_UseClassExceptionsFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_FrozenFlag")]
    pub static mut Py_FrozenFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_IgnoreEnvironmentFlag")]
    pub static mut Py_IgnoreEnvironmentFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_DontWriteBytecodeFlag")]
    pub static mut Py_DontWriteBytecodeFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    #[cfg_attr(PyPy, link_name = "PyPy_NoUserSiteDirectory")]
    pub static mut Py_NoUserSiteDirectory: c_int;
    #[deprecated(note = "Python 3.12")]
    pub static mut Py_UnbufferedStdioFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_HashRandomizationFlag")]
    pub static mut Py_HashRandomizationFlag: c_int;
    #[deprecated(note = "Python 3.12")]
    pub static mut Py_IsolatedFlag: c_int;
    #[cfg(windows)]
    #[deprecated(note = "Python 3.12")]
    pub static mut Py_LegacyWindowsFSEncodingFlag: c_int;
    #[cfg(windows)]
    #[deprecated(note = "Python 3.12")]
    pub static mut Py_LegacyWindowsStdioFlag: c_int;
}

extern "C" {
    #[cfg(Py_3_11)]
    pub fn Py_GETENV(name: *const c_char) -> *mut c_char;
}

#[cfg(not(Py_3_11))]
#[inline(always)]
pub unsafe fn Py_GETENV(name: *const c_char) -> *mut c_char {
    #[allow(deprecated)]
    if Py_IgnoreEnvironmentFlag != 0 {
        std::ptr::null_mut()
    } else {
        libc::getenv(name)
    }
}
