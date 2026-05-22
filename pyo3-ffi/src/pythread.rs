use core::ffi::c_ulong;

extern_libpython! {
    pub fn PyThread_get_thread_ident() -> c_ulong;
}
