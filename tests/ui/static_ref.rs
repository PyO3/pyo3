use pyo3::prelude::*;
use pyo3::types::PyList;

#[pyfunction]
fn static_ref(list: &'static Bound<'_, PyList>) -> usize {
    PyListMethods::len(list)
}

#[pyfunction]
fn static_py(list: &Bound<'static, PyList>) -> usize {
    PyListMethods::len(list)
}

fn main() {}
