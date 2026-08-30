use pyo3::prelude::*;

#[pyclass]
struct ClearWithoutTraverse;

#[pymethods]
impl ClearWithoutTraverse {
    fn __clear__(&mut self) {}
//~^ ERROR: an implementation of `__clear__` requires an implementation of `__traverse__`
}

fn main() {}
