#![cfg(all(Py_3_14, not(any(PyPy, GraalPy, RustPython, Py_LIMITED_API))))]

use pyo3::init_config::PyInitConfig;
use pyo3::prelude::Python;

#[test]
fn test_init_fail() {
    let mut config = PyInitConfig::default();
    // invalid allocator value should cause Python to fail to initialize reliably
    config.set_int(c"allocator", 999).unwrap();
    Python::initialize_from_init_config(config).unwrap_err();
}
