use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    fn method_with_invalid_self_type(slf: i32, py: Python<'_>, index: u32) {}
}

fn main() {}
