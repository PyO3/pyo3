#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::inspect::classes::InspectClass;

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

#[test]
fn simple_info() {
    let class_info = Simple::inspect();
    println!("Class:  {:?}", class_info);

    assert!(false)
}
