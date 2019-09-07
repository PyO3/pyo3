//! Objects related to PyBuffer and PyStr
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};

/// This is for confirming that PyBuffer does not cause memory leak
#[pyclass]
struct BytesExtractor {}

#[pymethods]
impl BytesExtractor {
    #[new]
    pub fn __new__(obj: &PyRawObject) {
        obj.init({ BytesExtractor {} });
    }

    pub fn to_vec(&mut self, bytes: &PyBytes) -> PyResult<usize> {
        let byte_vec: Vec<u8> = bytes.extract().unwrap();
        Ok(byte_vec.len())
    }

    pub fn to_str(&mut self, bytes: &PyString) -> PyResult<usize> {
        let byte_str: String = bytes.extract().unwrap();
        Ok(byte_str.len())
    }
}

#[pymodule]
pub fn buf_and_str(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<BytesExtractor>()?;
    Ok(())
}
