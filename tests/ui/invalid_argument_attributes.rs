use pyo3::prelude::*;

#[pyfunction]
fn invalid_attribute(#[pyo3(get)] param: String) {}

#[pyfunction]
fn from_py_with_no_value(#[pyo3(from_py_with)] param: String) {}

#[pyfunction]
fn from_py_with_string(#[pyo3("from_py_with")] param: String) {}

#[pyfunction]
fn from_py_with_value_not_a_string(#[pyo3(from_py_with = func)] param: String) {}

#[pyfunction]
fn from_py_with_repeated(#[pyo3(from_py_with = "func", from_py_with = "func")] param: String) {}

fn main() {}
