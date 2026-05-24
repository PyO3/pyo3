use pyo3::prelude::*;

#[pyclass]
struct InvalidNewReturn;

struct Invalid;

#[pymethods]
impl InvalidNewReturn {
    #[new]
    fn new() -> Invalid {
        Invalid
    }
}

fn main() {}
