use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

#[pyfunction(signature = ())]
fn none() {}

type Any<'py> = Bound<'py, PyAny>;
type Dict<'py> = Bound<'py, PyDict>;
type Tuple<'py> = Bound<'py, PyTuple>;

#[pyfunction(signature = (a, b = None, *, c = None))]
fn simple<'py>(
    a: Any<'py>,
    b: Option<Any<'py>>,
    c: Option<Any<'py>>,
) -> (Any<'py>, Option<Any<'py>>, Option<Any<'py>>) {
    (a, b, c)
}

#[pyfunction(signature = (a, b = None, *args, c = None))]
fn simple_args<'py>(
    a: Any<'py>,
    b: Option<Any<'py>>,
    args: Tuple<'py>,
    c: Option<Any<'py>>,
) -> (Any<'py>, Option<Any<'py>>, Tuple<'py>, Option<Any<'py>>) {
    (a, b, args, c)
}

#[pyfunction(signature = (a, b = None, c = None, **kwargs))]
fn simple_kwargs<'py>(
    a: Any<'py>,
    b: Option<Any<'py>>,
    c: Option<Any<'py>>,
    kwargs: Option<Dict<'py>>,
) -> (
    Any<'py>,
    Option<Any<'py>>,
    Option<Any<'py>>,
    Option<Dict<'py>>,
) {
    (a, b, c, kwargs)
}

#[pyfunction(signature = (a, b = None, *args, c = None, **kwargs))]
fn simple_args_kwargs<'py>(
    a: Any<'py>,
    b: Option<Any<'py>>,
    args: Tuple<'py>,
    c: Option<Any<'py>>,
    kwargs: Option<Dict<'py>>,
) -> (
    Any<'py>,
    Option<Any<'py>>,
    Tuple<'py>,
    Option<Any<'py>>,
    Option<Dict<'py>>,
) {
    (a, b, args, c, kwargs)
}

#[pyfunction(signature = (*args, **kwargs))]
fn args_kwargs<'py>(
    args: Tuple<'py>,
    kwargs: Option<Dict<'py>>,
) -> (Tuple<'py>, Option<Dict<'py>>) {
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
