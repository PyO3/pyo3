#![cfg(feature = "multiple-pymethods")]

use pyo3::prelude::*;
use pyo3::types::PyType;

#[macro_use]
#[path = "../src/tests/common.rs"]
mod common;

#[pyclass]
struct PyClassWithMultiplePyMethods {}

#[pymethods]
impl PyClassWithMultiplePyMethods {
    #[new]
    fn new() -> Self {
        Self {}
    }
}

#[pymethods]
impl PyClassWithMultiplePyMethods {
    fn __call__(&self) -> &'static str {
        "call"
    }
}

#[pymethods]
impl PyClassWithMultiplePyMethods {
    fn method(&self) -> &'static str {
        "method"
    }
}

#[pymethods]
impl PyClassWithMultiplePyMethods {
    #[classmethod]
    fn classmethod(_ty: &Bound<'_, PyType>) -> &'static str {
        "classmethod"
    }
}

#[pymethods]
impl PyClassWithMultiplePyMethods {
    #[staticmethod]
    fn staticmethod() -> &'static str {
        "staticmethod"
    }
}

#[pymethods]
impl PyClassWithMultiplePyMethods {
    #[classattr]
    fn class_attribute() -> &'static str {
        "class_attribute"
    }
}

#[pymethods]
impl PyClassWithMultiplePyMethods {
    #[classattr]
    const CLASS_ATTRIBUTE: &'static str = "CLASS_ATTRIBUTE";
}

#[test]
fn test_class_with_multiple_pymethods() {
    Python::with_gil(|py| {
        let cls = py.get_type::<PyClassWithMultiplePyMethods>();
        py_assert!(py, cls, "cls()() == 'call'");
        py_assert!(py, cls, "cls().method() == 'method'");
        py_assert!(py, cls, "cls.classmethod() == 'classmethod'");
        py_assert!(py, cls, "cls.staticmethod() == 'staticmethod'");
        py_assert!(py, cls, "cls.class_attribute == 'class_attribute'");
        py_assert!(py, cls, "cls.CLASS_ATTRIBUTE == 'CLASS_ATTRIBUTE'");
    })
}
