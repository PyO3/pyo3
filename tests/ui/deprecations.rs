#![deny(deprecated)]

use pyo3::prelude::*;

#[pyfunction(_opt = "None", x = "5")]
fn function_with_args(_opt: Option<i32>, _x: i32) {}

#[pyfunction]
fn function_with_required_after_option(_opt: Option<i32>, _x: i32) {}

#[pyclass]
struct MyClass;

#[pymethods]
impl MyClass {
    #[args(_opt = "None", x = "5")]
    fn function_with_args(&self, _opt: Option<i32>, _x: i32) {}

    #[args(_has_default = 1)]
    fn default_arg_before_required_deprecated(&self, _has_default: isize, _required: isize) {}
}

fn main() {
    function_with_required_after_option(None, 0);
    function_with_args(None, 0);
}
