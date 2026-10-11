use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyTuple};

#[pyfunction]
fn generic_function<T>(_value: T) {}
//~^ ERROR: Python functions cannot have generic type parameters

#[pyfunction]
fn impl_trait_function(_impl_trait: impl AsRef<PyAny>) {}
//~^ ERROR: Python functions cannot have `impl Trait` arguments

#[pyfunction]
fn wildcard_argument(_: i32) {}
//~^ ERROR: wildcard argument names are not supported

#[pyfunction]
fn destructured_argument((_a, _b): (i32, i32)) {}
//~^ ERROR: destructuring in arguments is not supported

#[pyfunction]
#[pyo3(signature=(*args))]
fn function_with_optional_args(args: Option<Bound<'_, PyTuple>>) {
    //~^ ERROR: args cannot be optional
    let _ = args;
}

#[pyfunction]
#[pyo3(signature=(**kwargs))]
fn function_with_required_kwargs(kwargs: Bound<'_, PyDict>) {
    //~^ ERROR: kwargs must be Option<_>
    let _ = kwargs;
}

#[pyfunction(pass_module)]
fn pass_module_but_no_arguments<'py>() {}
//~^ ERROR: expected `&PyModule` or `Py<PyModule>` as first argument with `pass_module`

#[pyfunction(pass_module)]
fn first_argument_not_module<'a, 'py>(
    _string: &str,
    //~^ ERROR: the trait bound `&str: From<&pyo3::Bound<'_, pyo3::types::PyModule>>` is not satisfied
    module: &'a Bound<'py, PyModule>,
) -> PyResult<Bound<'py, PyString>> {
    module.name()
}

fn main() {}
