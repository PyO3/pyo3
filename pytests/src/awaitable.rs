//! The following classes are examples of objects which implement Python's
//! awaitable protocol.
//!
//! Both IterAwaitable and FutureAwaitable will return a value immediately
//! when awaited, see guide examples related to pyo3-asyncio for ways
//! to suspend tasks and await results.

use pyo3::{prelude::*, pyclass::IterNextOutput};

#[pyclass]
#[derive(Debug)]
pub(crate) struct IterAwaitable {
    result: Option<PyResult<PyObject>>,
}

#[pymethods]
impl IterAwaitable {
    #[new]
    fn new(result: PyObject) -> Self {
        IterAwaitable {
            result: Some(Ok(result)),
        }
    }

    fn __await__(pyself: PyRef<'_, Self>) -> PyRef<'_, Self> {
        pyself
    }

    fn __iter__(pyself: PyRef<'_, Self>) -> PyRef<'_, Self> {
        pyself
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<IterNextOutput<PyObject, PyObject>> {
        match self.result.take() {
            Some(res) => match res {
                Ok(v) => Ok(IterNextOutput::Return(v)),
                Err(err) => Err(err),
            },
            _ => Ok(IterNextOutput::Yield(py.None())),
        }
    }
}

#[pyclass]
pub(crate) struct FutureAwaitable {
    #[pyo3(get, set, name = "_asyncio_future_blocking")]
    py_block: bool,
    result: Option<PyResult<PyObject>>,
}

#[pymethods]
impl FutureAwaitable {
    #[new]
    fn new(result: PyObject) -> Self {
        FutureAwaitable {
            py_block: false,
            result: Some(Ok(result)),
        }
    }

    fn __await__(pyself: PyRef<'_, Self>) -> PyRef<'_, Self> {
        pyself
    }

    fn __iter__(pyself: PyRef<'_, Self>) -> PyRef<'_, Self> {
        pyself
    }

    fn __next__(
        mut pyself: PyRefMut<'_, Self>,
    ) -> PyResult<IterNextOutput<PyRefMut<'_, Self>, PyObject>> {
        match pyself.result {
            Some(_) => match pyself.result.take().unwrap() {
                Ok(v) => Ok(IterNextOutput::Return(v)),
                Err(err) => Err(err),
            },
            _ => Ok(IterNextOutput::Yield(pyself)),
        }
    }
}

#[pymodule]
pub fn awaitable(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<IterAwaitable>()?;
    m.add_class::<FutureAwaitable>()?;
    Ok(())
}
