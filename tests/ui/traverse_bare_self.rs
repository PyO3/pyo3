use pyo3::prelude::*;
use pyo3::PyVisit;

#[pyclass]
struct TraverseTriesToTakePyRef {}

#[pymethods]
impl TraverseTriesToTakePyRef {
    fn __traverse__(slf: PyRef<Self>, visit: PyVisit) {}
}

fn main() {}
