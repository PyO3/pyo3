use std::os::raw::c_int;
#[cfg(not(PyPy))]
use std::os::raw::c_longlong;
#[cfg(target_os = "wasi")]
use std::os::raw::c_uint;
#[cfg(any(not(PyPy), windows))]
use std::os::raw::c_ulong;

#[cfg(all(not(PyPy), not(Py_3_13), not(windows)))]
pub const PY_TIMEOUT_MAX: c_longlong = c_longlong::MAX / 1000;
#[cfg(all(not(PyPy), not(Py_3_11), windows))]
pub const PY_TIMEOUT_MAX: c_longlong = (0xFFFFFFFF as c_longlong).saturating_mul(1000);
#[cfg(all(not(PyPy), Py_3_11, not(Py_3_13), windows))]
pub const PY_TIMEOUT_MAX: c_longlong = (0xFFFFFFFE as c_longlong).saturating_mul(1000);
#[cfg(all(not(any(PyPy, GraalPy)), Py_3_13))]
#[cfg_attr(windows, link(name = "pythonXY"))]
extern "C" {
    pub static PY_TIMEOUT_MAX: c_longlong;
}

#[cfg(not(PyPy))]
pub const PYTHREAD_INVALID_THREAD_ID: c_ulong = c_ulong::MAX;

// skipped _PyThread_at_fork_reinit (removed 3.13)

#[cfg(not(any(windows, target_os = "wasi")))]
type NATIVE_TSS_KEY_T = libc::pthread_key_t;
#[cfg(windows)]
type NATIVE_TSS_KEY_T = c_ulong;
#[cfg(target_os = "wasi")]
type NATIVE_TSS_KEY_T = c_uint;

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Py_tss_t {
    _is_initialized: c_int,
    _key: NATIVE_TSS_KEY_T,
}

impl Default for Py_tss_t {
    fn default() -> Self {
        Py_tss_NEEDS_INIT
    }
}

pub const Py_tss_NEEDS_INIT: Py_tss_t = Py_tss_t {
    _is_initialized: 0,
    _key: 0,
};
