use pyo3::prelude::*;
use pyo3::types::PyList;

#[pyfunction]
fn static_ref(list: &'static Bound<'_, PyList>) -> usize {
    list.len()
}

#[pyfunction]
fn static_py(list: &Bound<'static, PyList>) -> usize {
    list.len()
}

fn main() {}
