use pyo3::prelude::*;
use pyo3::types::PyBool;

#[pyclass(extends=PyBool)]
//~^ ERROR: pyclass `PyBool` cannot be subclassed
//~| ERROR: pyclass `PyBool` cannot be subclassed
struct ExtendsBool;

fn main() {}
