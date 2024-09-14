use pyo3::prelude::*;
use pyo3::types::PyBool;

#[pyclass(extends=PyBool)]
struct ExtendsBool;

fn main() {}
