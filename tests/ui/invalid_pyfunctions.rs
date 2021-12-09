use pyo3::prelude::*;

#[pyfunction]
fn generic_function<T>(value: T) {}

#[pyfunction]
fn impl_trait_function(impl_trait: impl AsRef<PyAny>) {}

#[pyfunction]
async fn async_function() {}

#[pyfunction]
fn required_arg_after_optional(optional: Option<isize>, required: isize) {}

fn main() {}
