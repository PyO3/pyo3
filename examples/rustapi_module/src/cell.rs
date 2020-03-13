//! Test for [?](https://github.com/PyO3/pyo3/issues/?)

use pyo3::prelude::*;

#[pyclass]
pub struct Mutable {}

#[pymethods]
impl Mutable {
    #[new]
    fn new() -> Self {
        Mutable {}
    }
    fn invalid_borrow(&mut self, _other: &Mutable) {}
    fn invalid_borrow_mut(&mut self, _other: &Mutable) {}
}

#[pymodule]
fn cell(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Mutable>()?;
    Ok(())
}
