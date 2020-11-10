use pyo3::class::iter::{IterNextOutput, PyIterProtocol};
use pyo3::prelude::*;

/// This is for demonstrating how to return a value from __next__
#[pyclass]
struct PyClassIter {
    count: usize,
}

#[pymethods]
impl PyClassIter {
    #[new]
    pub fn new() -> Self {
        PyClassIter { count: 0 }
    }
}

#[pyproto]
impl PyIterProtocol for PyClassIter {
    fn __next__(mut slf: PyRefMut<Self>) -> IterNextOutput<usize, &'static str> {
        if slf.count < 5 {
            slf.count += 1;
            IterNextOutput::Yield(slf.count)
        } else {
            IterNextOutput::Return("Ended")
        }
    }
}

#[pymodule]
pub fn pyclass_iter(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyClassIter>()?;
    Ok(())
}
