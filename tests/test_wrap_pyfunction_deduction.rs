use pyo3::{prelude::*, types::PyCFunction, wrap_pyfunction};

#[pyfunction]
fn f() {}

pub fn add_wrapped(wrapper: &impl Fn(Python) -> PyResult<&PyCFunction>) {
    let _ = wrapper;
}

#[test]
fn wrap_pyfunction_deduction() {
    add_wrapped(wrap_pyfunction!(f));
}
