use pyo3::prelude::*;

#[pyfunction]
fn generic_function<T>(value: T) {}

#[pyfunction]
fn impl_trait_function(impl_trait: impl AsRef<PyAny>) {}

#[pyfunction]
async fn async_function() {}

#[pyfunction]
fn wildcard_argument(_: i32) {}

#[pyfunction]
fn destructured_argument((a, b): (i32, i32)) {}

fn main() {}
