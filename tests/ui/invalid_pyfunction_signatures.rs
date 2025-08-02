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
    let _ = kwargs;
}

#[pyfunction]
#[pyo3(signature = (**kwargs_a, **kwargs_b))]
fn function_with_kwargs_after_kwargs(kwargs_a: Option<&PyDict>, kwargs_b: Option<&PyDict>) {
    let _ = kwargs_a;
    let _ = kwargs_b;
}

#[pyfunction(signature = (py))]
fn signature_contains_py(py: Python<'_>) {
    let _ = py;
}

#[pyclass]
struct MyClass;

#[pymethods]
impl MyClass {
    #[pyo3(signature = (**kwargs, *, *args, x))]
    fn multiple_errors_same_order(kwargs: Option<&PyDict>, args: &PyTuple, x: i32) {
        let _ = kwargs;
        let _ = args;
        let _ = x;
    }

    #[pyo3(signature = (**kwargs, *, *args, x))]
    fn multiple_errors_different_order(args: &PyTuple, x: i32, kwargs: Option<&PyDict>) {
        let _ = kwargs;
        let _ = args;
        let _ = x;
    }
}

fn main() {}
