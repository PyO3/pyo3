use pyo3::prelude::*;

#[doc = "This \0 contains a nul byte!"]
//~^ ERROR: Python doc may not contain nul byte, found nul at position 5
#[pyclass]
struct InvalidDocWithNulByte {}

fn main() {}
