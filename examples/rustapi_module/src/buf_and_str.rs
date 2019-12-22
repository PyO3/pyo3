//! Objects related to PyBuffer and PyStr
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
        let byte_vec: Vec<u8> = bytes.extract().unwrap();
        Ok(byte_vec.len())
    }

    pub fn from_str(&mut self, string: &PyString) -> PyResult<usize> {
        let rust_string: String = string.extract().unwrap();
        Ok(rust_string.len())
    }

    pub fn from_str_lossy(&mut self, string: &PyString) -> PyResult<usize> {
        let rust_string_lossy: String = string.to_string_lossy().to_string();
        Ok(rust_string_lossy.len())
    }
}

#[pymodule]
pub fn buf_and_str(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<BytesExtractor>()?;
    Ok(())
}
