#![cfg(not(Py_LIMITED_API))]

//! Objects related to PyBuffer and PyStr
use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyMemoryView, PyString};

/// This is for confirming that PyBuffer does not cause memory leak
#[pyclass]
struct BytesExtractor {}

#[pymethods]
impl BytesExtractor {
    #[new]
    pub fn __new__() -> Self {
        BytesExtractor {}
    }

    #[staticmethod]
    pub fn from_bytes(bytes: &PyBytes) -> PyResult<usize> {
        let byte_vec: Vec<u8> = bytes.extract()?;
        Ok(byte_vec.len())
    }

    #[staticmethod]
    pub fn from_str(string: &PyString) -> PyResult<usize> {
        let rust_string: String = string.extract()?;
        Ok(rust_string.len())
    }

    #[staticmethod]
    pub fn from_str_lossy(string: &PyString) -> usize {
        let rust_string_lossy: String = string.to_string_lossy().to_string();
        rust_string_lossy.len()
    }

    #[staticmethod]
    pub fn from_buffer(buf: &PyAny) -> PyResult<usize> {
        let buf = PyBuffer::<u8>::get(buf)?;
        Ok(buf.item_count())
    }
}

#[pyfunction]
fn return_memoryview(py: Python<'_>) -> PyResult<&PyMemoryView> {
    let bytes = PyBytes::new(py, b"hello world");
    let memoryview = PyMemoryView::from(bytes)?;
    Ok(memoryview)
}

#[pymodule]
pub fn buf_and_str(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<BytesExtractor>()?;
    m.add_function(wrap_pyfunction!(return_memoryview, m)?)?;
    Ok(())
}
