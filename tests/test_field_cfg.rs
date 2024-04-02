#![cfg(feature = "macros")]

use pyo3::prelude::*;

#[pyclass]
struct CfgClass {
    #[pyo3(get, set)]
    #[cfg(any())]
    pub a: u32,
    #[pyo3(get, set)]
    // This is always true
    #[cfg(any(
        target_family = "unix",
        target_family = "windows",
        target_family = "wasm"
    ))]
    pub b: u32,
}

#[test]
fn test_cfg() {
    Python::with_gil(|py| {
        let cfg = CfgClass { b: 3 };
        let py_cfg = Py::new(py, cfg).unwrap();
        assert!(py_cfg.bind(py).getattr("a").is_err());
        let b: u32 = py_cfg.bind(py).getattr("b").unwrap().extract().unwrap();
        assert_eq!(b, 3);
    });
}
