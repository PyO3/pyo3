use pyo3::prelude::*;

#[pyfunction]
fn generic_function<T>(value: T) {}

#[pyfunction]
fn impl_trait_function(impl_trait: impl AsRef<PyAny>) {}

fn main() {}
