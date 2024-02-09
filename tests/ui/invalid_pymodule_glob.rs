use pyo3::prelude::*;

#[pyfunction]
fn foo() -> usize {
    0
}

#[pymodule]
mod module {
    #[pyo3]
    use super::*;
}

fn main() {}
