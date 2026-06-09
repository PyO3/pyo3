use core::ffi::c_ulong;

extern_libpython! {
    #[cfg_attr(PyPy, link_name = "PyPyThread_get_thread_ident")]
    pub fn PyThread_get_thread_ident() -> c_ulong;
}
