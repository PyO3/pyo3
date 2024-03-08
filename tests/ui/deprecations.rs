#![deny(deprecated)]

use pyo3::prelude::*;
use pyo3::types::{PyString, PyType};

#[pyclass]
struct MyClass;

#[pymethods]
impl MyClass {
    #[__new__]
    fn new() -> Self {
        Self
    }

    #[classmethod]
    fn cls_method_gil_ref(_cls: &PyType) {}

    #[classmethod]
    fn cls_method_bound(_cls: &Bound<'_, PyType>) {}

    fn method_gil_ref(_slf: &PyCell<Self>) {}

    fn method_bound(_slf: &Bound<'_, Self>) {}
}

fn main() {}

#[pyfunction]
#[pyo3(pass_module)]
fn pyfunction_with_module<'py>(module: &Bound<'py, PyModule>) -> PyResult<Bound<'py, PyString>> {
    module.name()
}

#[pyfunction]
#[pyo3(pass_module)]
fn pyfunction_with_module_gil_ref(module: &PyModule) -> PyResult<&str> {
    module.name()
}

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
