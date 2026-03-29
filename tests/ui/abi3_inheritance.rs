use pyo3::exceptions::PyException;
use pyo3::prelude::*;

#[pyclass(extends=PyException)]
//~^ error: `PyException` cannot be subclassed
//~| error: `PyException` cannot be subclassed
#[derive(Clone)]
struct MyException {
    code: u32,
}

fn main() {}
