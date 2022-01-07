use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    fn __truediv__(&self, _py: Python) -> PyResult<()> {
        Ok(())
    }
}

fn main() {}
