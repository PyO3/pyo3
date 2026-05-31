//! The following classes are examples of objects which implement Python's
//! awaitable protocol.
//!
//! Both IterAwaitable and FutureAwaitable will return a value immediately
//! when awaited, see guide examples related to pyo3-async-runtimes for ways
//! to suspend tasks and await results.

use pyo3::prelude::*;

#[pymodule]
pub mod awaitable {
    use pyo3::exceptions::PyStopIteration;
    use pyo3::prelude::*;

    #[pyclass]
    #[derive(Debug)]
    pub(crate) struct IterAwaitable {
        result: Option<PyResult<Py<PyAny>>>,
    }

    #[pymethods]
    impl IterAwaitable {
        #[new]
        fn new(result: Py<PyAny>) -> Self {
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

        fn __next__(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
            match self.result.take() {
                Some(res) => match res {
                    Ok(v) => Err(PyStopIteration::new_err(v)),
                    Err(err) => Err(err),
                },
                _ => Ok(py.None()),
            }
        }

        fn send(&mut self, value: Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
            self.__next__(value.py())
        }

        #[pyo3(signature = (value, _a = None, _b = None))]
        fn throw(
            &mut self,
            value: Bound<'_, PyAny>,
            _a: Option<Bound<'_, PyAny>>,
            _b: Option<Bound<'_, PyAny>>,
        ) -> PyResult<Py<PyAny>> {
            self.__next__(value.py())
        }

        fn close(&self) {}
    }

    #[pyclass]
    pub(crate) struct FutureAwaitable {
        #[pyo3(get, set, name = "_asyncio_future_blocking")]
        py_block: bool,
        result: Option<PyResult<Py<PyAny>>>,
    }

    #[pymethods]
    impl FutureAwaitable {
        #[new]
        fn new(result: Py<PyAny>) -> Self {
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

        fn __next__(mut pyself: PyRefMut<'_, Self>) -> PyResult<PyRefMut<'_, Self>> {
            match pyself.result {
                Some(_) => match pyself.result.take().unwrap() {
                    Ok(v) => Err(PyStopIteration::new_err(v)),
                    Err(err) => Err(err),
                },
                _ => Ok(pyself),
            }
        }

        fn send<'py>(
            pyself: PyRefMut<'py, Self>,
            _value: Bound<'py, PyAny>,
        ) -> PyResult<PyRefMut<'py, Self>> {
            Self::__next__(pyself)
        }

        #[pyo3(signature = (_value, _a = None, _b = None))]
        fn throw<'py>(
            pyself: PyRefMut<'py, Self>,
            _value: Bound<'py, PyAny>,
            _a: Option<Bound<'py, PyAny>>,
            _b: Option<Bound<'py, PyAny>>,
        ) -> PyResult<PyRefMut<'py, Self>> {
            Self::__next__(pyself)
        }

        fn close(&self) {}
    }
}
