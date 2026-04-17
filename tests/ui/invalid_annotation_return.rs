use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (a) -> "int")]
//~^ ERROR: Return type annotation in the signature is only supported with the `experimental-inspect` feature
fn check(a: usize) -> usize {
    a
}

fn main() {}
