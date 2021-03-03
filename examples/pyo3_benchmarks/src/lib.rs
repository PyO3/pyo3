use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyo3::wrap_pyfunction;

#[pyfunction(args = "*", kwargs = "**")]
fn args_and_kwargs<'a>(
    args: &'a PyTuple,
    kwargs: Option<&'a PyDict>,
) -> (&'a PyTuple, Option<&'a PyDict>) {
    (args, kwargs)
}

#[pyfunction(a, b = 2, args = "*", c = 4, kwargs = "**")]
fn mixed_args<'a>(
    a: i32,
    b: i32,
    args: &'a PyTuple,
    c: i32,
    kwargs: Option<&'a PyDict>,
) -> (i32, i32, &'a PyTuple, i32, Option<&'a PyDict>) {
    (a, b, args, c, kwargs)
}

#[pyfunction]
fn no_args() {}

#[pymodule]
fn _pyo3_benchmarks(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(args_and_kwargs, m)?)?;
    m.add_function(wrap_pyfunction!(mixed_args, m)?)?;
    m.add_function(wrap_pyfunction!(no_args, m)?)?;
    Ok(())
}
