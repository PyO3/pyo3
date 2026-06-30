#![cfg(all(Py_3_14, not(any(PyPy, GraalPy, RustPython, Py_LIMITED_API))))]
#![allow(clippy::undocumented_unsafe_blocks, reason = "tests")]

use pyo3::init_config::InitConfig;

#[test]
fn test_initializes_interpreter() {
    let config = InitConfig::default();
    config.initialize().unwrap();
    assert_ne!(unsafe { pyo3::ffi::Py_IsInitialized() }, 0);
}
