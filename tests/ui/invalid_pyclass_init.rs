use pyo3::prelude::*;

#[pyclass]
struct InvalidInitReturn;

#[pymethods]
impl InvalidInitReturn {
    fn __init__(&self) -> i32 {
        //~^ ERROR: the trait bound `i32: pyo3::impl_::callback::IntoPyCallbackOutput<'_, i32>` is not satisfied
        0
    }
}

fn main() {}
