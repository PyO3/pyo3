#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::types::PyIterable;

#[pyclass]
struct MyIterable;

#[pymethods]
impl MyIterable {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(&mut self) -> Option<i32> {
        None
    }
}

#[test]
fn register_pyclass_as_iterable() {
    Python::attach(|py| {
        PyIterable::register::<MyIterable>(py).unwrap();
        let obj = Py::new(py, MyIterable).unwrap();
        assert!(obj.bind(py).cast::<PyIterable>().is_ok());
    });
}
