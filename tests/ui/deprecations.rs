#![deny(deprecated)]

use pyo3::prelude::*;

#[pyclass]
#[pyo3(text_signature = "()")]
struct MyClass;

#[pymethods]
impl MyClass {
    #[__new__]
    fn new() -> Self {
        Self
    }
}

fn main() {}
