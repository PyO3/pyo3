#![cfg(all(Py_3_14, not(any(PyPy, GraalPy, RustPython, Py_LIMITED_API))))]

use pyo3::init_config::PyInitConfig;
use pyo3::prelude::Python;

#[test]
#[ignore = "this doesn't seem to be a reliable way to make python fail to initialize"]
fn test_init_fail() {
    let mut config = PyInitConfig::default();
    config
        .set_str(c"prefix", c"/path/that/does/not/exit")
        .unwrap();
    Python::initialize_from_init_config(config).unwrap_err();
}
