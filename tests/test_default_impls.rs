#![cfg(feature = "macros")]

use pyo3::prelude::*;

mod test_utils;

// Test default generated __repr__.
#[pyclass(eq, eq_int)]
#[derive(PartialEq)]
enum TestDefaultRepr {
    Var,
}

#[test]
fn test_default_slot_exists() {
    Python::attach(|py| {
        let test_object = Py::new(py, TestDefaultRepr::Var).unwrap();
        py_assert!(
            py,
            test_object,
            "repr(test_object) == 'TestDefaultRepr.Var'"
        );
    })
}

#[pyclass(eq, eq_int)]
#[derive(PartialEq)]
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
    Python::attach(|py| {
        let test_object = Py::new(py, OverrideSlot::Var).unwrap();
        py_assert!(py, test_object, "repr(test_object) == 'overridden'");
    })
}
