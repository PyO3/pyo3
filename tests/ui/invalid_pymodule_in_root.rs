use pyo3::prelude::*;

#[pymodule]
#[path = "empty.rs"] // to silence error related to missing file
mod invalid_pymodule_in_root_module;

fn main() {}
