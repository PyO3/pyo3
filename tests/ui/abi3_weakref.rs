//! With abi3, weakref not supported until python 3.9 or greater
use pyo3::prelude::*;

#[pyclass(weakref)]
struct TestClass {}

fn main() {}
