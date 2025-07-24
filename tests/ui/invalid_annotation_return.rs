use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (a) -> "int")]
fn check(a: usize) -> usize {
    a
}

fn main() {}
