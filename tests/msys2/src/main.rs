unsafe extern "C" {
    fn py_is_initialized() -> std::ffi::c_int;
}

fn main() {
    #[rustfmt::skip]
    assert_eq!(
        unsafe { pyo3_ffi::Py_IsInitialized() },
        unsafe { py_is_initialized() }
    );
}
