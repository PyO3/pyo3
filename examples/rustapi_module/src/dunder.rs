use pyo3::prelude::*;

#[pymodule]
fn dunder(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Number>()?;
    Ok(())
}

#[pyclass]
pub struct Number {
    value: u32,
}

#[pymethods]
impl Number {
    #[new]
    fn new(obj: &PyRawObject, value: u32) {
        obj.init(Number { value })
    }

    /// Very basic add function
    fn __add__(&self, other: u32) -> PyResult<u32> {
        Ok(self.value + other)
    }
}
