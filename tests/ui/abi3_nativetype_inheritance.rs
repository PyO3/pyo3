use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass(extends=PyDict)]
struct TestClass {}

fn main() {}
