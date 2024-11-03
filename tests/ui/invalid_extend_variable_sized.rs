use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple, PyType};

#[pyclass(extends=PyType)]
#[derive(Default)]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[pyo3(signature = (*_args, **_kwargs))]
    fn __init__(&mut self, _args: &Bound<'_, PyTuple>, _kwargs: Option<&Bound<'_, PyDict>>) {}
}

fn main() {}
