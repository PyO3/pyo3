use pyo3::prelude::*;
use pyo3::types::PyList;

#[pyfunction]
//~^ ERROR: borrowed data escapes outside of function
//~| ERROR: temporary value dropped while borrowed
fn static_ref(list: &'static Bound<'_, PyList>) -> usize {
    list.len()
}

#[pyfunction]
//~^ ERROR: borrowed data escapes outside of function
fn static_py(list: &Bound<'static, PyList>) -> usize {
    list.len()
}

fn main() {}
