use std::os::raw::{c_char, c_int};

#[cfg_attr(windows, link(name="pythonXY"))] extern "C" {
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_DebugFlag")]
    pub static mut Py_DebugFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_VerboseFlag")]
    pub static mut Py_VerboseFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_InteractiveFlag")]
    pub static mut Py_InteractiveFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_InspectFlag")]
    pub static mut Py_InspectFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_OptimizeFlag")]
    pub static mut Py_OptimizeFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_NoSiteFlag")]
    pub static mut Py_NoSiteFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_BytesWarningFlag")]
    pub static mut Py_BytesWarningFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_UseClassExceptionsFlag")]
    pub static mut Py_UseClassExceptionsFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_FrozenFlag")]
    pub static mut Py_FrozenFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_TabcheckFlag")]
    pub static mut Py_TabcheckFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_UnicodeFlag")]
    pub static mut Py_UnicodeFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_IgnoreEnvironmentFlag")]
    pub static mut Py_IgnoreEnvironmentFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_DivisionWarningFlag")]
    pub static mut Py_DivisionWarningFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_DontWriteBytecodeFlag")]
    pub static mut Py_DontWriteBytecodeFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_NoUserSiteDirectory")]
    pub static mut Py_NoUserSiteDirectory: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}__PyPy_QnewFlag")]
    pub static mut _Py_QnewFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_Py3kWarningFlag")]
    pub static mut Py_Py3kWarningFlag: c_int;
    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_HashRandomizationFlag")]
    pub static mut Py_HashRandomizationFlag: c_int;

    #[cfg_attr(PyPy, link_name="\u{1}_PyPy_FatalError")]
    pub fn Py_FatalError(message: *const c_char);
}