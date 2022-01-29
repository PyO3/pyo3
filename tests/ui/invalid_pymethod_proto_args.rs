use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    fn __truediv__(&self) -> PyResult<()> {
        Ok(())
    }
}

fn main() {}
