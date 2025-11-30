use pyo3::prelude::*;

#[pyclass]
struct InvalidInitReturn;

#[pymethods]
impl InvalidInitReturn {
    fn __init__(&self) -> i32 {
        0
    }
}

fn main() {}
