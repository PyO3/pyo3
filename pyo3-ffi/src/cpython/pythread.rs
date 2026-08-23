use core::ffi::c_int;

// skipped PY_TIMEOUT_MAX
// skipped PYTHREAD_INVALID_THREAD_ID

#[cfg(windows)]
type NATIVE_TSS_KEY_T = core::ffi::c_ulong;

#[cfg(not(windows))]
type NATIVE_TSS_KEY_T = libc::pthread_key_t;

#[repr(C)]
pub struct Py_tss_t {
    _is_initialized: c_int,
    _key: NATIVE_TSS_KEY_T,
}

pub const Py_tss_NEEDS_INIT: Py_tss_t = Py_tss_t {
    _is_initialized: 0,
    _key: 0,
};
