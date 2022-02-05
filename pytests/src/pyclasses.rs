use pyo3::iter::IterNextOutput;
use pyo3::prelude::*;

#[pyclass]
struct EmptyClass {}

#[pymethods]
impl EmptyClass {
    #[new]
    fn new() -> Self {
        EmptyClass {}
    }
}

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

    fn __next__(&mut self) -> IterNextOutput<usize, &'static str> {
        if self.count < 5 {
            self.count += 1;
            IterNextOutput::Yield(self.count)
        } else {
            IterNextOutput::Return("Ended")
        }
    }
}

#[pymodule]
pub fn pyclasses(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<EmptyClass>()?;
    m.add_class::<PyClassIter>()?;
    Ok(())
}
