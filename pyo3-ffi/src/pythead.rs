use core::ffi::{c_int, c_void};

// TODO use non-opaque type if not(PY_LIMITED_API)
opaque_struct!(pub Py_tss_t);

extern_libpython! {
    pub fn PyThread_tss_alloc() -> *mut Py_tss_t;
    pub fn PyThread_tss_free(key: *mut Py_tss_t);

    pub fn PyThread_tss_is_created(key: *mut Py_tss_t) -> c_int;
    pub fn PyThread_tss_create(key: *mut Py_tss_t) -> c_int;
    pub fn PyThread_tss_delete(key: *mut Py_tss_t);
    pub fn PyThread_tss_set(key: *mut Py_tss_t, value: *mut c_void);
    pub fn PyThread_tss_get(key: *mut Py_tss_t) -> *mut c_void;
}
