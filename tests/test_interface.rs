#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::interface::GetClassInfo;
use pyo3::interface::GetClassFields;

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
    let class_info = Simple::info();
    let fields_info = Simple::fields_info();
    println!("Class:  {:?}", class_info);
    println!("Fields: {:?}", fields_info);
}
