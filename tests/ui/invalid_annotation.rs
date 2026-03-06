use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (a: "int"))]
//~^ ERROR: Type annotations in the signature is only supported with the `experimental-inspect` feature
fn check(a: usize) -> usize {
    a
}

fn main() {}
