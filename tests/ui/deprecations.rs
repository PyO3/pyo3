#![deny(deprecated)]

use pyo3::prelude::*;

#[pyclass(gc)]
struct DeprecatedGc;

#[pyfunction(_opt = "None", x = "5")]
fn function_with_args(_opt: Option<i32>, _x: i32) {}

#[pyclass]
struct MyClass;

#[pymethods]
impl MyClass {
    #[args(_opt = "None", x = "5")]
    fn function_with_args(&self, _opt: Option<i32>, _x: i32) {}
}

fn main() {
    function_with_args(None, 0);
}
