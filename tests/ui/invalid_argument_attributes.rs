use pyo3::prelude::*;

#[pyfunction]
fn invalid_attribute(#[pyo3(get)] _param: String) {}

#[pyfunction]
fn from_py_with_no_value(#[pyo3(from_py_with)] _param: String) {}

#[pyfunction]
fn from_py_with_string(#[pyo3("from_py_with")] _param: String) {}

#[pyfunction]
fn from_py_with_value_not_a_string(#[pyo3(from_py_with = func)] _param: String) {}

#[pyfunction]
fn from_py_with_repeated(#[pyo3(from_py_with = "func", from_py_with = "func")] _param: String) {}

fn main() {}
