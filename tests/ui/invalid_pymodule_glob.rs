use pyo3::prelude::*;

#[pyfunction]
fn foo() -> usize {
    0
}

#[pymodule]
mod module {
    #[pymodule_export]
    use super::*;
}

fn main() {}
