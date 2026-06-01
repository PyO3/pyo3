use pyo3::prelude::*;

#[pyclass]
struct InvalidNewReturn;

struct Invalid;

#[pymethods]
impl InvalidNewReturn {
    #[new]
    fn new() -> Invalid {
        //~^ ERROR: `Invalid` cannot be used as the return value for `#[new]` methods
        Invalid
    }
}

fn main() {}
