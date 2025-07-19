//! These tests are located in a separate file because they cause conflicting implementation
//! errors, which means other errors such as typechecking errors are not reported.

use pyo3::prelude::*;

#[pyclass]
struct TwoNew {}

#[pymethods]
impl TwoNew {
    #[new]
    fn new_1() -> Self {
        Self {}
    }

    #[new]
    fn new_2() -> Self {
        Self {}
    }
}

#[pyclass]
struct DuplicateMethod {}

#[pymethods]
impl DuplicateMethod {
    #[pyo3(name = "func")]
    fn func_a(&self) {}

    #[pyo3(name = "func")]
    fn func_b(&self) {}
}

fn main() {}
