use pyo3::prelude::*;
use pyo3::types::PyList;

#[pyfunction]
fn static_ref(list: Py2<'static, PyList>) -> usize {
    list.len()
}

fn main() {}
