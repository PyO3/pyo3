use pyo3::prelude::*;
use pyo3::types::PyList;

#[pyclass]
struct MyClass {
    list: &'static PyList,
}

#[pymethods]
impl MyClass {
    #[new]
    fn new(list: &'static PyList) -> Self {
        Self { list }
    }
}

fn main() {}
