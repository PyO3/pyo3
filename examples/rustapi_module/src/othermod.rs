//! https://github.com/PyO3/pyo3/issues/233
//!
//! The code below just tries to use the most important code generation paths

use pyo3::prelude::*;

#[pyclass]
pub struct ModClass {
    _somefield: String,
}

#[pymethods]
impl ModClass {
    fn noop(&self, x: usize) -> usize {
        x
    }
}

#[pyfunction]
fn double(x: i32) -> i32 {
    x*2
}

#[pymodinit]
fn datetime(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_function!(double))?;
    m.add_class::<ModClass>()?;
    Ok(())
}