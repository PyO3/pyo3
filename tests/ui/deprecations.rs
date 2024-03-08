#![deny(deprecated)]

use pyo3::prelude::*;

#[pyclass]
struct MyClass;

#[pymethods]
impl MyClass {
    #[__new__]
    fn new() -> Self {
        Self
    }
}

fn main() {}

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule]
fn module_gil_ref(m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)?;
    Ok(())
}

#[pymodule]
fn module_gil_ref_with_explicit_py_arg(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)?;
    Ok(())
}

#[pymodule]
fn module_bound(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)?;
    Ok(())
}

#[pymodule]
fn module_bound_with_explicit_py_arg(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)?;
    Ok(())
}

#[pymodule]
fn module_bound_by_value(m: Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, &m)?)?;
    Ok(())
}
