use pyo3::prelude::*;

#[pyclass]
pub struct Probe {}

#[pymethods]
impl Probe {
    #[new]
    fn new() -> Self {
        Self {}
    }
}

#[pymodule]
fn probe(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Probe>()?;
    Ok(())
}

fn main() {}
