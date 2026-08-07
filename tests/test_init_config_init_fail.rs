#![cfg(all(Py_3_14, not(any(PyPy, GraalPy, RustPython, Py_LIMITED_API))))]

use pyo3::init_config::InitConfig;

#[test]
fn test_init_fail() {
    let mut config = InitConfig::default();
    config
        .set_str(c"prefix", c"/path/that/does/not/exit")
        .unwrap();
    config.initialize().unwrap_err();
}
