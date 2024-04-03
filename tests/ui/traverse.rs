use pyo3::prelude::*;
use pyo3::PyTraverseError;
use pyo3::PyVisit;

#[pyclass]
struct TraverseTriesToTakePyRef {}

#[pymethods]
impl TraverseTriesToTakePyRef {
    fn __traverse__(slf: PyRef<Self>, visit: PyVisit) -> Result<(), PyTraverseError> {
        Ok(())
    }
}

#[pyclass]
struct TraverseTriesToTakePyRefMut {}

#[pymethods]
impl TraverseTriesToTakePyRefMut {
    fn __traverse__(slf: PyRefMut<Self>, visit: PyVisit) -> Result<(), PyTraverseError> {
        Ok(())
    }
}

#[pyclass]
struct TraverseTriesToTakeBound {}

#[pymethods]
impl TraverseTriesToTakeBound {
    fn __traverse__(slf: Bound<'_, Self>, visit: PyVisit) -> Result<(), PyTraverseError> {
        Ok(())
    }
}

#[pyclass]
struct TraverseTriesToTakeMutSelf {}

#[pymethods]
impl TraverseTriesToTakeMutSelf {
    fn __traverse__(&mut self, visit: PyVisit) -> Result<(), PyTraverseError> {
        Ok(())
    }
}

#[pyclass]
struct TraverseTriesToTakeSelf {}

#[pymethods]
impl TraverseTriesToTakeSelf {
    fn __traverse__(&self, visit: PyVisit) -> Result<(), PyTraverseError> {
        Ok(())
    }
}

#[pyclass]
struct Class;

#[pymethods]
impl Class {
    fn __traverse__(&self, py: Python<'_>, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        Ok(())
    }

    fn __clear__(&mut self) {}
}

fn main() {}
