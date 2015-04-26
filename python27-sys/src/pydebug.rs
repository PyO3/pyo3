use libc::{c_char, c_int};

#[link(name = "python2.7")]
extern "C" {
    pub static mut Py_DebugFlag: c_int;
    pub static mut Py_VerboseFlag: c_int;
    pub static mut Py_InteractiveFlag: c_int;
    pub static mut Py_InspectFlag: c_int;
    pub static mut Py_OptimizeFlag: c_int;
    pub static mut Py_NoSiteFlag: c_int;
    pub static mut Py_BytesWarningFlag: c_int;
    pub static mut Py_UseClassExceptionsFlag: c_int;
    pub static mut Py_FrozenFlag: c_int;
    pub static mut Py_TabcheckFlag: c_int;
    pub static mut Py_UnicodeFlag: c_int;
    pub static mut Py_IgnoreEnvironmentFlag: c_int;
    pub static mut Py_DivisionWarningFlag: c_int;
    pub static mut Py_DontWriteBytecodeFlag: c_int;
    pub static mut Py_NoUserSiteDirectory: c_int;
    pub static mut _Py_QnewFlag: c_int;
    pub static mut Py_Py3kWarningFlag: c_int;
    pub static mut Py_HashRandomizationFlag: c_int;

    pub fn Py_FatalError(message: *const c_char);
}

