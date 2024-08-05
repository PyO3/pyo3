use pyo3::prelude::*;

#[pyclass]
struct DeprecatedMethodContainer {}

#[pymethods]
impl DeprecatedMethodContainer {
    #[pyo3(deprecated = "deprecated __traverse__")]
    fn __traverse__(&self, _visit: pyo3::gc::PyVisit<'_>) -> Result<(), pyo3::PyTraverseError> {
        Ok(())
    }
}

#[pymethods]
impl DeprecatedMethodContainer {
    #[classattr]
    #[pyo3(deprecated = "deprecated class attr")]
    fn deprecated_class_attr() -> i32 {
        5
    }
}

#[pymethods]
impl DeprecatedMethodContainer {
    #[pyo3(deprecated = "first deprecatec")]
    #[pyo3(deprecated = "second deprecated")]
    fn function(&self) {}
}

fn main() {}
