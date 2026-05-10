use pyo3::prelude::*;

#[pyclass(immutable_type)]
//~^ ERROR: `immutable_type` requires Python >= 3.10 (or >= 3.14 when using the `abi3` feature)
struct ImmutableType {}

fn main() {}
