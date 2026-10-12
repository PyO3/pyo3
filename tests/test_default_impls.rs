#![cfg(feature = "macros")]

use pyo3::prelude::*;

mod test_utils;

// Test default generated __repr__.
#[pyclass(eq, eq_int)]
#[derive(PartialEq)]
enum TestDefaultRepr {
    Var,
}

#[pymethods]
impl TestDefaultRepr {
    #[cfg(any())]
    fn __repr__(&self) -> &'static str {
        "disabled"
    }
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

    fn __int__(&self) -> i32 {
        42
    }
}

#[test]
fn test_override_slot() {
    Python::attach(|py| {
        let test_object = Py::new(py, OverrideSlot::Var).unwrap();
        py_assert!(py, test_object, "repr(test_object) == 'overridden'");
        py_assert!(py, test_object, "int(test_object) == 42");
        #[cfg(feature = "multiple-pymethods")]
        py_assert!(
            py,
            test_object,
            "test_object.method_from_another_block() == 123"
        );
    })
}
