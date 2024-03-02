#![cfg(feature = "macros")]

use pyo3::{prelude::*, types::PyCFunction};

#[pyfunction]
fn f() {}

pub fn add_wrapped(wrapper: &impl Fn(Python<'_>) -> PyResult<&PyCFunction>) {
    let _ = wrapper;
}

#[test]
fn wrap_pyfunction_deduction() {
    add_wrapped(wrap_pyfunction!(f));
}

pub fn add_wrapped_bound(wrapper: &impl Fn(Python<'_>) -> PyResult<Bound<'_, PyCFunction>>) {
    let _ = wrapper;
}

#[test]
fn wrap_pyfunction_deduction_bound() {
    add_wrapped_bound(wrap_pyfunction_bound!(f));
}
