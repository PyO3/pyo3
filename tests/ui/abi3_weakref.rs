use pyo3::prelude::*;

#[pyclass(weakref)]
//~^ ERROR: `weakref` requires Python >= 3.9 when using the `abi3` feature
struct TestClass {}

fn main() {}
