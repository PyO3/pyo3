#![deny(deprecated)]

use pyo3::prelude::*;

#[pyclass]
struct MyClass;

#[pymethods]
impl MyClass {
    #[__new__]
    fn new() -> Self {
        Self
    }
}

fn main() {}
