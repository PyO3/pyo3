use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

#[pyfunction(signature = ())]
fn none() {}

#[pyfunction(signature = (a, b = None, *, c = None))]
fn simple<'a>(
    a: &'a PyAny,
    b: Option<&'a PyAny>,
    c: Option<&'a PyAny>,
) -> (&'a PyAny, Option<&'a PyAny>, Option<&'a PyAny>) {
    (a, b, c)
}

#[pyfunction(signature = (a, b = None, *args, c = None))]
fn simple_args<'a>(
    a: &'a PyAny,
    b: Option<&'a PyAny>,
    args: &'a PyTuple,
    c: Option<&'a PyAny>,
) -> (&'a PyAny, Option<&'a PyAny>, &'a PyTuple, Option<&'a PyAny>) {
    (a, b, args, c)
}

#[pyfunction(signature = (a, b = None, c = None, **kwargs))]
fn simple_kwargs<'a>(
    a: &'a PyAny,
    b: Option<&'a PyAny>,
    c: Option<&'a PyAny>,
    kwargs: Option<&'a PyDict>,
) -> (
    &'a PyAny,
    Option<&'a PyAny>,
    Option<&'a PyAny>,
    Option<&'a PyDict>,
) {
    (a, b, c, kwargs)
}

#[pyfunction(signature = (a, b = None, *args, c = None, **kwargs))]
fn simple_args_kwargs<'a>(
    a: &'a PyAny,
    b: Option<&'a PyAny>,
    args: &'a PyTuple,
    c: Option<&'a PyAny>,
    kwargs: Option<&'a PyDict>,
) -> (
    &'a PyAny,
    Option<&'a PyAny>,
    &'a PyTuple,
    Option<&'a PyAny>,
    Option<&'a PyDict>,
) {
    (a, b, args, c, kwargs)
}

#[pyfunction(signature = (*args, **kwargs))]
fn args_kwargs<'a>(
    args: &'a PyTuple,
    kwargs: Option<&'a PyDict>,
) -> (&'a PyTuple, Option<&'a PyDict>) {
    (args, kwargs)
}

#[pymodule]
pub fn pyfunctions(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(none, m)?)?;
    m.add_function(wrap_pyfunction!(simple, m)?)?;
    m.add_function(wrap_pyfunction!(simple_args, m)?)?;
    m.add_function(wrap_pyfunction!(simple_kwargs, m)?)?;
    m.add_function(wrap_pyfunction!(simple_args_kwargs, m)?)?;
    m.add_function(wrap_pyfunction!(args_kwargs, m)?)?;
    Ok(())
}
