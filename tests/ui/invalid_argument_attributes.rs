use pyo3::prelude::*;

#[pyfunction]
fn invalid_attribute(#[pyo3(get)] _param: String) {}

#[pyfunction]
fn from_py_with_no_value(#[pyo3(from_py_with)] _param: String) {}

#[pyfunction]
fn from_py_with_string(#[pyo3("from_py_with")] _param: String) {}

#[pyfunction]
fn from_py_with_value_not_found(#[pyo3(from_py_with = func)] _param: String) {}

#[pyfunction]
fn from_py_with_repeated(#[pyo3(from_py_with = "func", from_py_with = "func")] _param: String) {}

fn bytes_from_py(bytes: &Bound<'_, pyo3::types::PyBytes>) -> Vec<u8> {
    bytes.as_bytes().to_vec()
}

#[pyfunction]
fn f(#[pyo3(from_py_with = "bytes_from_py")] _bytes: Vec<u8>) {}

fn main() {}
