use pyo3::prelude::*;

#[pyclass(dict)]
//~^ ERROR: `dict` requires Python >= 3.9 when using the `abi3` feature
struct TestClass {}

fn main() {}
