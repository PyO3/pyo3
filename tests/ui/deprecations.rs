#![deny(deprecated)]

use pyo3::prelude::*;

#[pyclass(gc)]
struct DeprecatedGc;

#[pyfunction]
fn function_with_required_after_option(_opt: Option<i32>, _x: i32) {}

fn main() {
    function_with_required_after_option(None, 0);
}
