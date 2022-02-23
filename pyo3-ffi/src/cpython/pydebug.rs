use std::os::raw::c_int;

#[cfg(not(Py_LIMITED_API))]
#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPy_DebugFlag")]
    pub static mut Py_DebugFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_VerboseFlag")]
    pub static mut Py_VerboseFlag: c_int;
    pub static mut Py_QuietFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_InteractiveFlag")]
    pub static mut Py_InteractiveFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_InspectFlag")]
    pub static mut Py_InspectFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_OptimizeFlag")]
    pub static mut Py_OptimizeFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_NoSiteFlag")]
    pub static mut Py_NoSiteFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_BytesWarningFlag")]
    pub static mut Py_BytesWarningFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_UseClassExceptionsFlag")]
    pub static mut Py_UseClassExceptionsFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_FrozenFlag")]
    pub static mut Py_FrozenFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_IgnoreEnvironmentFlag")]
    pub static mut Py_IgnoreEnvironmentFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_DontWriteBytecodeFlag")]
    pub static mut Py_DontWriteBytecodeFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_NoUserSiteDirectory")]
    pub static mut Py_NoUserSiteDirectory: c_int;
    pub static mut Py_UnbufferedStdioFlag: c_int;
    #[cfg_attr(PyPy, link_name = "PyPy_HashRandomizationFlag")]
    pub static mut Py_HashRandomizationFlag: c_int;
    pub static mut Py_IsolatedFlag: c_int;
    #[cfg(windows)]
    pub static mut Py_LegacyWindowsStdioFlag: c_int;
}
