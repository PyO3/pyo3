use pyo3::prelude::*;

#[pyfunction]
fn generic_function<T>(value: T) {}

#[pyfunction]
fn impl_trait_function(impl_trait: impl AsRef<PyAny>) {}

#[pyfunction]
fn wildcard_argument(_: i32) {}

#[pyfunction]
fn destructured_argument((a, b): (i32, i32)) {}

#[pyfunction]
fn function_with_required_after_option(_opt: Option<i32>, _x: i32) {}

fn main() {}
