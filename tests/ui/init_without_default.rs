use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    fn __init__(&self) {}
}

fn main() {}
