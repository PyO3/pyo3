use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    fn method_with_invalid_self_type(slf: i32, py: Python<'_>, index: u32) {}
}

#[pyclass(frozen)]
struct MyClass2 {}

#[pymethods]
impl MyClass2 {
    fn method_with_invalid_self_type(&mut self, py: Python<'_>, index: u32) {}
}

fn main() {}
