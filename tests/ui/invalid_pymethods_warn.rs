use pyo3::prelude::*;

#[pyclass]
struct WarningMethodContainer {}

#[pymethods]
impl WarningMethodContainer {
    #[pyo3(warn(message = "warn on __traverse__"))]
//~^ ERROR: __traverse__ cannot be used with #[pyo3(warn)]
    fn __traverse__(&self) {}
}

#[pymethods]
impl WarningMethodContainer {
    #[classattr]
    #[pyo3(warn(message = "warn for class attr"))]
//~^ ERROR: #[classattr] cannot be used with #[pyo3(warn)]
    fn a_class_attr(_py: pyo3::Python<'_>) -> i64 {
        5
    }
}

fn main() {}
