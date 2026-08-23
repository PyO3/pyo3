use core::ffi::{c_int, c_void};

// skipped PyThread_type_lock
// skipped PyLockStatus
// skipped PyThread_init_thread
// skipped PyThread_start_new_thread
// skipped PyThread_exit_thread
// skipped PyThread_get_thread_ident
// skipped PyThread_get_thread_native_id
// skipped PyThread_allocate_lock
// skipped PyThread_free_lock
// skipped PyThread_acquire_lock
// skipped WAIT_LOCK
// skipped NOWAIT_LOCK
// skipped PY_TIMEOUT_T
// skipped PyThread_acquire_lock_timed
// skipped PyThread_release_lock
// skipped PyThread_get_stacksize
// skipped PyThread_set_stacksize
// skipped PyThread_GetInfo
// skipped PyThread_create_key
// skipped PyThread_delete_key
// skipped PyThread_set_key_value
// skipped PyThread_get_key_value
// skipped PyThread_delete_key_value
// skipped PyThread_ReInitTLS

#[cfg(Py_LIMITED_API)]
opaque_struct!(pub Py_tss_t);

#[cfg(not(Py_LIMITED_API))]
#[repr(C)]
pub struct Py_tss_t {
    _is_initialized: c_int,
    _key: NATIVE_TSS_KEY_T,
}

#[cfg(not(Py_LIMITED_API))]
pub const Py_tss_NEEDS_INIT: Py_tss_t = Py_tss_t {
    _is_initialized: 0,
    _key: 0,
};

#[cfg(all(windows, not(Py_LIMITED_API)))]
type NATIVE_TSS_KEY_T = core::ffi::c_ulong;

#[cfg(all(not(windows), not(Py_LIMITED_API)))]
type NATIVE_TSS_KEY_T = libc::pthread_key_t;

extern_libpython! {
    pub fn PyThread_tss_alloc() -> *mut Py_tss_t;
    pub fn PyThread_tss_free(key: *mut Py_tss_t);

    pub fn PyThread_tss_is_created(key: *mut Py_tss_t) -> c_int;
    pub fn PyThread_tss_create(key: *mut Py_tss_t) -> c_int;
    pub fn PyThread_tss_delete(key: *mut Py_tss_t);
    pub fn PyThread_tss_set(key: *mut Py_tss_t, value: *mut c_void) -> c_int;
    pub fn PyThread_tss_get(key: *mut Py_tss_t) -> *mut c_void;
}
