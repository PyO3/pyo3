use crate::object::PyObject;
#[cfg(not(PyPy))]
use std::os::raw::c_longlong;
use std::os::raw::{c_int, c_ulong, c_void};

pub type PyThread_type_lock = *mut c_void;
// skipped PyThread_type_sema (removed 3.9)

#[cfg(not(PyPy))]
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PyLockStatus {
    PY_LOCK_FAILURE = 0,
    PY_LOCK_ACQUIRED = 1,
    PY_LOCK_INTR,
}

extern "C" {
    pub fn PyThread_init_thread();
    pub fn PyThread_start_new_thread(
        arg1: Option<unsafe extern "C" fn(*mut c_void)>,
        arg2: *mut c_void,
    ) -> c_ulong;

    // skipped PyThread_exit_thread (deprecated 3.14)

    pub fn PyThread_get_thread_ident() -> c_ulong;

    #[cfg(all(
        not(PyPy),
        any(
            target_os = "ios",
            target_os = "macos",
            target_os = "tvos",
            target_os = "watchos",
            target_os = "android",
            target_os = "linux",
            target_os = "windows",
            target_os = "freebsd",
            target_os = "openbsd",
            target_os = "netbsd",
            all(not(GraalPy), Py_3_12, target_os = "dragonfly"),
            target_os = "aix"
        )
    ))]
    pub fn PyThread_get_thread_native_id() -> c_ulong;

    pub fn PyThread_allocate_lock() -> PyThread_type_lock;
    pub fn PyThread_free_lock(arg1: PyThread_type_lock);
    pub fn PyThread_acquire_lock(arg1: PyThread_type_lock, arg2: c_int) -> c_int;
}

pub const WAIT_LOCK: c_int = 1;
pub const NOWAIT_LOCK: c_int = 0;

#[cfg(not(PyPy))]
pub type PY_TIMEOUT_T = c_longlong;

extern "C" {
    #[cfg(not(PyPy))]
    pub fn PyThread_acquire_lock_timed(
        arg1: PyThread_type_lock,
        microseconds: PY_TIMEOUT_T,
        intr_flag: c_int,
    ) -> PyLockStatus;

    pub fn PyThread_release_lock(arg1: PyThread_type_lock);

    #[cfg(not(PyPy))]
    pub fn PyThread_get_stacksize() -> usize;

    #[cfg(not(PyPy))]
    pub fn PyThread_set_stacksize(arg1: usize) -> c_int;

    #[cfg(not(PyPy))]
    pub fn PyThread_GetInfo() -> *mut PyObject;

    // skipped PyThread_create_key (deprecated 3.7)
    // skipped PyThread_delete_key (deprecated 3.7)
    // skipped PyThread_set_key_value (deprecated 3.7)
    // skipped PyThread_get_key_value (deprecated 3.7)
    // skipped PyThread_delete_key_value (deprecated 3.7)
    // skipped PyThread_ReInitTLS (deprecated 3.7)
}

#[cfg(Py_LIMITED_API)]
opaque_struct!(Py_tss_t);
#[cfg(not(Py_LIMITED_API))]
use crate::cpython::pythread::Py_tss_t;

extern "C" {
    pub fn PyThread_tss_alloc() -> *mut Py_tss_t;
    pub fn PyThread_tss_free(key: *mut Py_tss_t);
    pub fn PyThread_tss_is_created(key: *mut Py_tss_t) -> c_int;
    pub fn PyThread_tss_create(key: *mut Py_tss_t) -> c_int;
    pub fn PyThread_tss_delete(key: *mut Py_tss_t);
    pub fn PyThread_tss_set(key: *mut Py_tss_t, value: *mut c_void) -> c_int;
    pub fn PyThread_tss_get(key: *mut Py_tss_t) -> *mut c_void;
}
