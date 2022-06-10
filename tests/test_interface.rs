#![cfg(feature = "macros")]

use pyo3::prelude::*;

mod common;

#[pyclass]
struct Simple {}

#[pymethods]
impl Simple {
    #[new]
    fn new() -> Self {
        Self {}
    }

    fn plus_one(&self, a: usize) -> usize {
        a + 1
    }
}

#[test]
fn compiles() {
    // Nothing to do: if we reach this point, the compilation was successful :)
}
