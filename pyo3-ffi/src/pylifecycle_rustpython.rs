use crate::pytypedefs::PyThreadState;
use crate::rustpython_runtime;

use libc::wchar_t;
use std::ffi::{c_char, c_int};

#[inline]
pub unsafe fn Py_Initialize() {
    rustpython_runtime::initialize();
}

#[inline]
pub unsafe fn Py_InitializeEx(_initsigs: c_int) {
    rustpython_runtime::initialize();
}

#[inline]
pub unsafe fn Py_Finalize() {
    rustpython_runtime::finalize();
}

#[inline]
pub unsafe fn Py_FinalizeEx() -> c_int {
    rustpython_runtime::finalize();
    0
}

#[inline]
pub unsafe fn Py_IsInitialized() -> c_int {
    rustpython_runtime::is_initialized().into()
}

#[inline]
pub unsafe fn Py_NewInterpreter() -> *mut PyThreadState {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn Py_EndInterpreter(_tstate: *mut PyThreadState) {}

#[inline]
pub unsafe fn Py_AtExit(_func: Option<extern "C" fn()>) -> c_int {
    0
}

#[inline]
pub unsafe fn Py_Exit(code: c_int) -> ! {
    std::process::exit(code)
}

#[inline]
pub unsafe fn Py_Main(_argc: c_int, _argv: *mut *mut wchar_t) -> c_int {
    -1
}

#[inline]
pub unsafe fn Py_BytesMain(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
    -1
}

#[inline]
pub unsafe fn Py_SetProgramName(_name: *const wchar_t) {}

#[inline]
pub unsafe fn Py_GetProgramName() -> *mut wchar_t {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn Py_SetPythonHome(_home: *const wchar_t) {}

#[inline]
pub unsafe fn Py_GetPythonHome() -> *mut wchar_t {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn Py_GetProgramFullPath() -> *mut wchar_t {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn Py_GetPrefix() -> *mut wchar_t {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn Py_GetExecPrefix() -> *mut wchar_t {
    std::ptr::null_mut()
}

#[inline]
pub unsafe fn Py_GetPath() -> *mut wchar_t {
    std::ptr::null_mut()
}

#[cfg(not(Py_3_13))]
#[inline]
pub unsafe fn Py_SetPath(_path: *const wchar_t) {}

#[inline]
pub unsafe fn Py_GetVersion() -> *const c_char {
    c"RustPython".as_ptr()
}

#[inline]
pub unsafe fn Py_GetPlatform() -> *const c_char {
    c"rustpython".as_ptr()
}

#[inline]
pub unsafe fn Py_GetCopyright() -> *const c_char {
    c"RustPython".as_ptr()
}

#[inline]
pub unsafe fn Py_GetCompiler() -> *const c_char {
    c"rustc".as_ptr()
}

#[inline]
pub unsafe fn Py_GetBuildInfo() -> *const c_char {
    c"rustpython backend".as_ptr()
}

pub type PyOS_sighandler_t = unsafe extern "C" fn(arg1: c_int);

#[inline]
pub unsafe fn PyOS_getsig(_sig: c_int) -> PyOS_sighandler_t {
    unsafe extern "C" fn noop(_sig: c_int) {}
    noop
}

#[inline]
pub unsafe fn PyOS_setsig(_sig: c_int, handler: PyOS_sighandler_t) -> PyOS_sighandler_t {
    handler
}

#[cfg(Py_3_11)]
#[unsafe(no_mangle)]
pub static Py_Version: std::ffi::c_ulong = 0;

#[cfg(Py_3_13)]
#[inline]
pub unsafe fn Py_IsFinalizing() -> c_int {
    0
}
