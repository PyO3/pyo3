use pyo3::prelude::*;

#[doc = "This \0 contains a nul byte!"]
#[pyclass]
struct InvalidDocWithNulByte {}

fn main() {}
