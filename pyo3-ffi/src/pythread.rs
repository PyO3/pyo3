use core::ffi::{c_int, c_void};

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
