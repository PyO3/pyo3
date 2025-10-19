use pyo3::exceptions::PyException;
use pyo3::prelude::*;

#[pyclass(extends=PyException)]
#[derive(Clone)]
struct MyException {
    code: u32,
}

fn main() {}
