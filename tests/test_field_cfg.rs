#![cfg(feature = "macros")]

use pyo3::prelude::*;

#[pyclass]
struct CfgClass {
    #[pyo3(get, set)]
    #[cfg(any())]
    pub a: u32,
    #[pyo3(get, set)]
    #[cfg(all())]
    pub b: u32,
}

#[test]
fn test_cfg() {
    Python::with_gil(|py| {
        let cfg = CfgClass { b: 3 };
        let py_cfg = Py::new(py, cfg).unwrap();
        assert!(py_cfg.as_ref(py).getattr("a").is_err());
        let b: u32 = py_cfg.as_ref(py).getattr("b").unwrap().extract().unwrap();
        assert_eq!(b, 3);
    });
}
