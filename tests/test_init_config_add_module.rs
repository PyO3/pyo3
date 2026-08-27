#![cfg(all(Py_3_14, not(any(PyPy, GraalPy, RustPython, Py_LIMITED_API))))]
#![cfg(feature = "macros")]
#![allow(clippy::undocumented_unsafe_blocks, reason = "tests")]

use pyo3::add_module_to_init_config;
use pyo3::init_config::PyInitConfig;
use pyo3::prelude::*;

#[test]
fn test_add_module() {
    let mut config = PyInitConfig::default();
    add_module_to_init_config!(config, m).unwrap();
    Python::initialize_from_init_config(config).unwrap();
    Python::attach(|py| {
        let m = py.import("m").unwrap();
        let get_42 = m.getattr("get_42").unwrap();
        let forty_two = get_42.call0().unwrap().extract::<i32>().unwrap();
        assert_eq!(42, forty_two);
    });
}

#[pymodule]
mod m {
    use pyo3::prelude::*;

    #[pyfunction]
    fn get_42() -> i32 {
        42
    }
}
