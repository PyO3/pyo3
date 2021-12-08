#![cfg(not(Py_LIMITED_API))]

//! Objects related to PyBuffer and PyStr
use pyo3::buffer::PyBuffer;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};

/// This is for confirming that PyBuffer does not cause memory leak
#[pyclass]
struct BytesExtractor {}

#[pymethods]
impl BytesExtractor {
    #[new]
    pub fn __new__() -> Self {
        BytesExtractor {}
    }

    pub fn from_bytes(&mut self, bytes: &PyBytes) -> PyResult<usize> {
        let byte_vec: Vec<u8> = bytes.extract()?;
        Ok(byte_vec.len())
    }

    pub fn from_str(&mut self, string: &PyString) -> PyResult<usize> {
        let rust_string: String = string.extract()?;
        Ok(rust_string.len())
    }

    pub fn from_str_lossy(&mut self, string: &PyString) -> PyResult<usize> {
        let rust_string_lossy: String = string.to_string_lossy().to_string();
        Ok(rust_string_lossy.len())
    }

    pub fn from_buffer(&mut self, buf: &PyAny) -> PyResult<usize> {
        let buf = PyBuffer::<u8>::get(buf)?;
        Ok(buf.item_count())
    }
}

#[pymodule]
pub fn buf_and_str(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<BytesExtractor>()?;
    Ok(())
}
