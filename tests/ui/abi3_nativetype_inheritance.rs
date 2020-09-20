//! With abi3, we cannot inherite native types.
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyclass(extends=PyDict)]
struct TestClass {}

fn main() {}
