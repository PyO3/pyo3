use pyo3::prelude::*;

#[pymodule]
#[path = "empty.rs"] // to silence error related to missing file
mod invalid_pymodule_in_root_module;
//~^ ERROR: file modules in proc macro input are unstable
//~| ERROR: `#[pymodule]` can only be used on inline modules

fn main() {}
