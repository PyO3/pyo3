use pyo3::prelude::*;
use std::sync::atomic::AtomicPtr;

#[pyfunction]
fn invalid_pyfunction_argument(arg: AtomicPtr<()>) {
    //~^ ERROR: `AtomicPtr<()>` cannot be used as a Python function argument
    //~| ERROR: `AtomicPtr<()>` cannot be used as a Python function argument
    //~| ERROR: `AtomicPtr<()>` cannot be used as a Python function argument
    let _ = arg;
}

#[pyclass(skip_from_py_object)]
#[derive(Clone)]
struct Foo;

#[pyfunction]
fn skip_from_py_object_without_custom_from_py_object(arg: Foo) {
    //~^ ERROR: `Foo` cannot be used as a Python function argument
    let _ = arg;
}

fn main() {}
