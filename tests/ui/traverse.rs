use pyo3::prelude::*;
use pyo3::PyVisit;
use pyo3::PyTraverseError;

#[pyclass]
struct TraverseTriesToTakePyRef {}

#[pymethods]
impl TraverseTriesToTakePyRef {
    fn __traverse__(slf: PyRef<Self>, visit: PyVisit) {}
}

#[pyclass]
struct Class;

#[pymethods]
impl Class {
    fn __traverse__(&self, py: Python<'_>, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        Ok(())
    }

    fn __clear__(&mut self) {
    }
}


fn main() {}
