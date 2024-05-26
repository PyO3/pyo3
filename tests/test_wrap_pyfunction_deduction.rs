#![cfg(feature = "macros")]

use pyo3::{prelude::*, types::PyCFunction};

#[pyfunction]
fn f() {}

#[cfg(feature = "gil-refs")]
pub fn add_wrapped(wrapper: &impl Fn(Python<'_>) -> PyResult<&PyCFunction>) {
    let _ = wrapper;
}

#[test]
fn wrap_pyfunction_deduction() {
    #[allow(deprecated)]
    #[cfg(feature = "gil-refs")]
    add_wrapped(wrap_pyfunction!(f));
    #[cfg(not(feature = "gil-refs"))]
    add_wrapped_bound(wrap_pyfunction!(f));
}

pub fn add_wrapped_bound(wrapper: &impl Fn(Python<'_>) -> PyResult<Bound<'_, PyCFunction>>) {
    let _ = wrapper;
}

#[test]
fn wrap_pyfunction_deduction_bound() {
    add_wrapped_bound(wrap_pyfunction_bound!(f));
}
