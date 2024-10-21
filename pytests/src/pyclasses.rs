use std::{thread, time};

use pyo3::exceptions::{PyStopIteration, PyValueError};
use pyo3::prelude::*;
use pyo3::types::PyType;

#[pyclass]
struct EmptyClass {}

#[pymethods]
impl EmptyClass {
    #[new]
    fn new() -> Self {
        EmptyClass {}
    }

    fn method(&self) {}

    fn __len__(&self) -> usize {
        0
    }
}

/// This is for demonstrating how to return a value from __next__
#[pyclass]
#[derive(Default)]
struct PyClassIter {
    count: usize,
}

#[pymethods]
impl PyClassIter {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    fn __next__(&mut self) -> PyResult<usize> {
        if self.count < 5 {
            self.count += 1;
            Ok(self.count)
        } else {
            Err(PyStopIteration::new_err("Ended"))
        }
    }
}

#[pyclass]
#[derive(Default)]
struct PyClassThreadIter {
    count: usize,
}

#[pymethods]
impl PyClassThreadIter {
    #[new]
    pub fn new() -> Self {
        Default::default()
    }

    fn __next__(&mut self, py: Python<'_>) -> usize {
        let current_count = self.count;
        self.count += 1;
        if current_count == 0 {
            py.allow_threads(|| thread::sleep(time::Duration::from_millis(100)));
        }
        self.count
    }
}

/// Demonstrates a base class which can operate on the relevant subclass in its constructor.
#[pyclass(subclass)]
#[derive(Clone, Debug)]
struct AssertingBaseClass;

#[pymethods]
impl AssertingBaseClass {
    #[new]
    #[classmethod]
    fn new(cls: &Bound<'_, PyType>, expected_type: Bound<'_, PyType>) -> PyResult<Self> {
        if !cls.is(&expected_type) {
            return Err(PyValueError::new_err(format!(
                "{:?} != {:?}",
                cls, expected_type
            )));
        }
        Ok(Self)
    }
}

#[pyclass]
struct ClassWithoutConstructor;

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
#[pyclass(dict)]
struct ClassWithDict;

#[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
#[pymethods]
impl ClassWithDict {
    #[new]
    fn new() -> Self {
        ClassWithDict
    }
}

#[pymodule(supports_free_threaded = true)]
pub fn pyclasses(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<EmptyClass>()?;
    m.add_class::<PyClassIter>()?;
    m.add_class::<PyClassThreadIter>()?;
    m.add_class::<AssertingBaseClass>()?;
    m.add_class::<ClassWithoutConstructor>()?;
    #[cfg(any(Py_3_10, not(Py_LIMITED_API)))]
    m.add_class::<ClassWithDict>()?;

    Ok(())
}
