use std::ffi::c_char;
#[cfg(all(not(Py_LIMITED_API), not(Py_3_11)))]
use std::ffi::c_int;

#[cfg(all(not(Py_LIMITED_API), not(Py_3_11)))]
#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    #[cfg_attr(PyPy, link_name = "PyPy_IgnoreEnvironmentFlag")]
    pub static mut Py_IgnoreEnvironmentFlag: c_int;
}

extern "C" {
    #[cfg(Py_3_11)]
    pub fn Py_GETENV(name: *const c_char) -> *mut c_char;
}

#[cfg(not(Py_3_11))]
#[inline(always)]
pub unsafe fn Py_GETENV(name: *const c_char) -> *mut c_char {
    if Py_IgnoreEnvironmentFlag != 0 {
        std::ptr::null_mut()
    } else {
        libc::getenv(name)
    }
}
