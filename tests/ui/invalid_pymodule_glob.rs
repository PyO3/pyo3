#![allow(unused_imports)]

use pyo3::prelude::*;

#[pyfunction]
fn foo() -> usize {
    0
}

#[pymodule]
mod module {
    #[pymodule_export]
    use super::*;
//~^ ERROR: #[pymodule] cannot import glob statements
}

fn main() {}
