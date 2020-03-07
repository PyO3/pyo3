use pyo3::prelude::*;

#[pyfunction(a = 5, b)]
fn pos_after_kw(py: Python, a: i32, b: i32) -> PyObject {
    [a.to_object(py), vararg.into()].to_object(py)
}

#[pyfunction(a, "*", b)]
fn pos_after_separator(py: Python, a: i32, b: i32) -> PyObject {
    [a.to_object(py), vararg.into()].to_object(py)
}

#[pyfunction(kwargs = "**", a = 5)]
fn kw_after_kwargs(py: Python, kwargs: &PyDict, a: i32) -> PyObject {
    [a.to_object(py), vararg.into()].to_object(py)
}

fn main() {}
