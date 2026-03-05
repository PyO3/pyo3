use pyo3::prelude::*;
use std::sync::atomic::AtomicPtr;

#[pyfunction]
fn invalid_pyfunction_argument(arg: AtomicPtr<()>) {
    let _ = arg;
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct Foo;

#[pyfunction]
fn skip_from_py_object_without_custom_from_py_object(arg: Foo) {
    let _ = arg;
}

fn main() {}
