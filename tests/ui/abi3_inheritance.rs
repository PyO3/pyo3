use pyo3::exceptions::PyException;
use pyo3::prelude::*;

#[pyclass(extends=PyException)]
//~^ ERROR: `PyException` cannot be subclassed
//~| ERROR: `PyException` cannot be subclassed
#[derive(Clone)]
struct MyException {
    code: u32,
}

fn main() {}
