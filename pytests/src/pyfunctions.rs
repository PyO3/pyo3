use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

#[pyfunction]
fn none() {}

#[pyfunction(b = "\"bar\"", "*", c = "None")]
fn simple<'a>(a: i32, b: &'a str, c: Option<&'a PyDict>) -> (i32, &'a str, Option<&'a PyDict>) {
    (a, b, c)
}

#[pyfunction(b = "\"bar\"", args = "*", c = "None")]
fn simple_args<'a>(
    a: i32,
    b: &'a str,
    c: Option<&'a PyDict>,
    args: &'a PyTuple,
) -> (i32, &'a str, &'a PyTuple, Option<&'a PyDict>) {
    (a, b, args, c)
}

#[pyfunction(b = "\"bar\"", c = "None", kwargs = "**")]
fn simple_kwargs<'a>(
    a: i32,
    b: &'a str,
    c: Option<&'a PyDict>,
    kwargs: Option<&'a PyDict>,
) -> (i32, &'a str, Option<&'a PyDict>, Option<&'a PyDict>) {
    (a, b, c, kwargs)
}

#[pyfunction(a, b = "\"bar\"", args = "*", c = "None", kwargs = "**")]
fn simple_args_kwargs<'a>(
    a: i32,
    b: &'a str,
    args: &'a PyTuple,
    c: Option<&'a PyDict>,
    kwargs: Option<&'a PyDict>,
) -> (
    i32,
    &'a str,
    &'a PyTuple,
    Option<&'a PyDict>,
    Option<&'a PyDict>,
) {
    (a, b, args, c, kwargs)
}

#[pyfunction(args = "*", kwargs = "**")]
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
