use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    fn method_with_invalid_self_type(_slf: i32, _py: Python<'_>, _index: u32) {}
}

fn main() {}
