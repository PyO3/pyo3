use pyo3::prelude::*;
use std::sync::atomic::AtomicPtr;

#[pyfunction]
fn invalid_pyfunction_argument(arg: AtomicPtr<()>) {
    let _ = arg;
}

fn main() {}
