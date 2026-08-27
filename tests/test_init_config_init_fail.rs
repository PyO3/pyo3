#![cfg(all(Py_3_14, not(any(PyPy, GraalPy, RustPython, Py_LIMITED_API))))]

use pyo3::init_config::InitConfig;
use pyo3::prelude::Python;

#[test]
fn test_init_fail() {
    let mut config = InitConfig::default();
    config
        .set_str(c"prefix", c"/path/that/does/not/exit")
        .unwrap();
    Python::initialize_from_init_config(config).unwrap();
}
