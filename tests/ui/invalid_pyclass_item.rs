use pyo3::prelude::*;

#[pyclass]
fn foo() {}
//~^ ERROR: #[pyclass] only supports structs and enums.

fn main() {}
