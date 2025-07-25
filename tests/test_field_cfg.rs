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

#[pyclass(eq, eq_int)]
#[derive(PartialEq)]
enum CfgSimpleEnum {
    #[cfg(any())]
    DisabledVariant,
    #[cfg(not(any()))]
    EnabledVariant,
}

#[test]
fn test_cfg() {
    Python::attach(|py| {
        let cfg = CfgClass { b: 3 };
        let py_cfg = Py::new(py, cfg).unwrap();
        assert!(py_cfg.bind(py).getattr("a").is_err());
        let b: u32 = py_cfg.bind(py).getattr("b").unwrap().extract().unwrap();
        assert_eq!(b, 3);
    });
}

#[test]
fn test_cfg_simple_enum() {
    Python::attach(|py| {
        let simple = py.get_type::<CfgSimpleEnum>();
        pyo3::py_run!(
            py,
            simple,
            r#"
            assert hasattr(simple, "EnabledVariant")
            assert not hasattr(simple, "DisabledVariant")
        "#
        );
    })
}
