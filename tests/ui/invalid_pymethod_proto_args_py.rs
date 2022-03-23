use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    fn __truediv__(&self, _py: Python<'_>) -> PyResult<()> {
        Ok(())
    }
}

fn main() {}
