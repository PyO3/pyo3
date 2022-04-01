use pyo3::prelude::*;

#[pyfunction(a = 5, b)]
fn pos_after_kw(py: Python<'_>, a: i32, b: i32) -> PyObject {
    [a.to_object(py), vararg.into()].to_object(py)
}

#[pyfunction(kwargs = "**", a = 5)]
fn kw_after_kwargs(py: Python<'_>, kwargs: &PyDict, a: i32) -> PyObject {
    [a.to_object(py), vararg.into()].to_object(py)
}

#[pyfunction(a, "*", b, "/", c)]
fn pos_only_after_kw_only(py: Python<'_>, a: i32, b: i32, c: i32) -> i32 {
    a + b + c
}

#[pyfunction(a, args="*", "/", b)]
fn pos_only_after_args(py: Python<'_>, a: i32, args: Vec<i32>, b: i32) -> i32 {
    a + b + c
}

#[pyfunction(a, kwargs="**", "/", b)]
fn pos_only_after_kwargs(py: Python<'_>, a: i32, args: Vec<i32>, b: i32) -> i32 {
    a + b
}

#[pyfunction(kwargs = "**", "*", a)]
fn kw_only_after_kwargs(py: Python<'_>, kwargs: &PyDict, a: i32) -> PyObject {
    [a.to_object(py), vararg.into()].to_object(py)
}

fn main() {}
