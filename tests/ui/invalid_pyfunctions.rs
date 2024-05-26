use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyTuple};

#[pyfunction]
fn generic_function<T>(_value: T) {}

#[pyfunction]
fn impl_trait_function(_impl_trait: impl AsRef<PyAny>) {}

#[pyfunction]
fn wildcard_argument(_: i32) {}

#[pyfunction]
fn destructured_argument((_a, _b): (i32, i32)) {}

#[pyfunction]
fn function_with_required_after_option(_opt: Option<i32>, _x: i32) {}

#[pyfunction]
#[pyo3(signature=(*args))]
fn function_with_optional_args(args: Option<Bound<'_, PyTuple>>) {
    let _ = args;
}

#[pyfunction]
#[pyo3(signature=(**kwargs))]
fn function_with_required_kwargs(kwargs: Bound<'_, PyDict>) {
    let _ = kwargs;
}

#[pyfunction(pass_module)]
fn pass_module_but_no_arguments<'py>() {}

#[pyfunction(pass_module)]
fn first_argument_not_module<'a, 'py>(
    _string: &str,
    module: &'a Bound<'py, PyModule>,
) -> PyResult<Bound<'py, PyString>> {
    module.name()
}

fn main() {}
