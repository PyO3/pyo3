//! With abi3, dict not supported until python 3.9 or greater
use pyo3::prelude::*;

#[pyclass(dict)]
struct TestClass {}

fn main() {}
