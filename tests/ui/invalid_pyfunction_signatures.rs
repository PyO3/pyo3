use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

#[pyfunction]
#[pyo3(signature = ())]
fn function_with_one_argument_empty_signature(_x: i32) {}

#[pyfunction]
#[pyo3(signature = (x))]
fn function_with_one_entry_signature_no_args() {}

#[pyfunction]
#[pyo3(signature = (x))]
fn function_with_incorrect_argument_names(y: i32) {
    let _ = y;
}

#[pyfunction(x)]
#[pyo3(signature = (x))]
fn function_with_both_args_and_signature(x: i32) {
    let _ = x;
}

#[pyfunction]
#[pyo3(signature = (*, *args))]
fn function_with_args_after_args_sep(args: &PyTuple) {
    let _ = args;
}

#[pyfunction]
#[pyo3(signature = (*, *))]
fn function_with_args_sep_after_args_sep() {}

#[pyfunction]
#[pyo3(signature = (**kwargs, *args))]
fn function_with_args_after_kwargs(kwargs: Option<&PyDict>, args: &PyTuple) {
    let _ = args;
}

#[pyfunction]
#[pyo3(signature = (**kwargs_a, **kwargs_b))]
fn function_with_kwargs_after_kwargs(kwargs_a: Option<&PyDict>, kwargs_b: Option<&PyDict>) {
    let _ = kwargs_a;
    let _ = kwargs_b;
}

#[pyclass]
struct MyClass;

#[pymethods]
impl MyClass {
    #[args(x)]
    #[pyo3(signature = (x))]
    fn method_with_both_args_and_signature(&self, x: i32) {
        let _ = x;
    }
}

fn main() {}
