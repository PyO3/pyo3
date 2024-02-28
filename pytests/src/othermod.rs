//! <https://github.com/PyO3/pyo3/issues/233>
//!
//! The code below just tries to use the most important code generation paths

use pyo3::prelude::*;

#[pyclass]
pub struct ModClass {
    _somefield: String,
}

#[pymethods]
impl ModClass {
    #[new]
    fn new() -> Self {
        ModClass {
            _somefield: String::from("contents"),
        }
    }

    fn noop(&self, x: usize) -> usize {
        x
    }
}

#[pyfunction]
fn double(x: i32) -> i32 {
    x * 2
}

#[pymodule]
pub fn othermod(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)?;

    m.add_class::<ModClass>()?;

    m.add("USIZE_MIN", usize::min_value())?;
    m.add("USIZE_MAX", usize::max_value())?;

    Ok(())
}
