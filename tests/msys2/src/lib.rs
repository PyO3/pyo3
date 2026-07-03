use std::ffi::c_int;

unsafe extern "C" {
    fn py_is_initialized() -> c_int;
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn pyo3_msys2_link_test_entry() -> c_int {
    pyo3_ffi::Py_IsInitialized() + py_is_initialized()
}
