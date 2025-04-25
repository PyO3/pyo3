//! The following classes are examples of objects which implement Python's
//! awaitable protocol.
//!
//! Both IterAwaitable and FutureAwaitable will return a value immediately
//! when awaited, see guide examples related to pyo3-async-runtimes for ways
//! to suspend tasks and await results.

use pyo3::exceptions::PyStopIteration;
use pyo3::prelude::*;

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

    fn __await__<'a, 'py>(pyself: PyRef<'a, 'py, Self>) -> PyRef<'a, 'py, Self> {
        pyself
    }

    fn __iter__<'a, 'py>(pyself: PyRef<'a, 'py, Self>) -> PyRef<'a, 'py, Self> {
        pyself
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        match self.result.take() {
            Some(res) => match res {
                Ok(v) => Err(PyStopIteration::new_err(v)),
                Err(err) => Err(err),
            },
            _ => Ok(py.None()),
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

    fn __await__<'a, 'py>(pyself: PyRef<'a, 'py, Self>) -> PyRef<'a, 'py, Self> {
        pyself
    }

    fn __iter__<'a, 'py>(pyself: PyRef<'a, 'py, Self>) -> PyRef<'a, 'py, Self> {
        pyself
    }

    fn __next__<'a, 'py>(mut pyself: PyRefMut<'a, 'py, Self>) -> PyResult<PyRefMut<'a, 'py, Self>> {
        match pyself.result {
            Some(_) => match pyself.result.take().unwrap() {
                Ok(v) => Err(PyStopIteration::new_err(v)),
                Err(err) => Err(err),
            },
            _ => Ok(pyself),
        }
    }
}

#[pymodule(gil_used = false)]
pub fn awaitable(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<IterAwaitable>()?;
    m.add_class::<FutureAwaitable>()?;
    Ok(())
}
