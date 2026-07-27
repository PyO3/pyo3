use core::ffi::c_ulong;

extern_libpython! {
    #[cfg(not(PyPy))]
    pub fn PyThread_get_thread_ident() -> c_ulong;
}
