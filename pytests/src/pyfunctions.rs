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

#[pyfunction(signature = (a, /, b))]
fn positional_only<'py>(a: Any<'py>, b: Any<'py>) -> (Any<'py>, Any<'py>) {
    (a, b)
}

#[pyfunction(signature = (a = false, b = 0, c = 0.0, d = ""))]
fn with_typed_args(a: bool, b: u64, c: f64, d: &str) -> (bool, u64, f64, &str) {
    (a, b, c, d)
}

#[pyfunction(signature = (a: "int", *_args: "str", _b: "int | None" = None, **_kwargs: "bool") -> "int")]
fn with_custom_type_annotations<'py>(
    a: Any<'py>,
    _args: Tuple<'py>,
    _b: Option<Any<'py>>,
    _kwargs: Option<Dict<'py>>,
) -> Any<'py> {
    a
}

#[pymodule]
pub mod pyfunctions {
    #[pymodule_export]
    use super::{
        args_kwargs, none, positional_only, simple, simple_args, simple_args_kwargs, simple_kwargs,
        with_custom_type_annotations, with_typed_args,
    };
}
