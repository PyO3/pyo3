#![deny(deprecated)]

use pyo3::prelude::*;

#[pyclass(subclass)]
struct Base;

#[pyclass(extends=Base)]
struct Sub1;

#[pymethods]
impl Sub1 {
    #[new]
    fn new() -> (Sub1, Base) {
        (Sub1, Base)
    }
}

#[pyclass(extends=Base)]
struct Sub2;

#[pymethods]
impl Sub2 {
    #[new]
    fn new() -> PyResult<(Sub1, Base)> {
        Ok((Sub1, Base))
    }
}

fn main() {}
