#![cfg(feature = "macros")]

use pyo3::prelude::*;

mod common;

// Test default generated __repr__.
#[pyclass]
enum TestDefaultRepr {
    Var,
}

#[test]
fn test_default_slot_exists() {
    Python::with_gil(|py| {
        let test_object = Py::new(py, TestDefaultRepr::Var).unwrap();
        py_assert!(
            py,
            test_object,
            "repr(test_object) == 'TestDefaultRepr.Var'"
        );
    })
}

#[pyclass]
enum OverrideSlot {
    Var,
}

#[pymethods]
impl OverrideSlot {
    fn __repr__(&self) -> &str {
        "overridden"
    }
}

#[test]
fn test_override_slot() {
    Python::with_gil(|py| {
        let test_object = Py::new(py, OverrideSlot::Var).unwrap();
        py_assert!(py, test_object, "repr(test_object) == 'overridden'");
    })
}
