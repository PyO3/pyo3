//! Python coroutine implementation, used notably when wrapping `async fn`
//! with `#[pyfunction]`/`#[pymethods]`.
use std::{
    any::Any,
    future::Future,
    panic,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use futures_util::FutureExt;
use pyo3_macros::{pyclass, pymethods};

use crate::{
    coroutine::waker::AsyncioWaker,
    exceptions::{PyAttributeError, PyRuntimeError, PyStopIteration},
    panic::PanicException,
    pyclass::IterNextOutput,
    types::{PyIterator, PyString},
    IntoPy, Py, PyAny, PyErr, PyObject, PyResult, Python,
};

mod waker;

const COROUTINE_REUSED_ERROR: &str = "cannot reuse already awaited coroutine";

type FutureOutput = Result<PyResult<PyObject>, Box<dyn Any + Send>>;

/// Python coroutine wrapping a [`Future`].
#[pyclass(crate = "crate")]
pub struct Coroutine {
    name: Option<Py<PyString>>,
    qualname_prefix: Option<&'static str>,
    future: Option<Pin<Box<dyn Future<Output = FutureOutput> + Send>>>,
    waker: Option<Arc<AsyncioWaker>>,
}

impl Coroutine {
    ///  Wrap a future into a Python coroutine.
    ///
    /// Coroutine `send` polls the wrapped future, ignoring the value passed
    /// (should always be `None` anyway).
    ///
    /// `Coroutine `throw` drop the wrapped future and reraise the exception passed
    pub(crate) fn new<F, T, E>(
        name: Option<Py<PyString>>,
        qualname_prefix: Option<&'static str>,
        future: F,
    ) -> Self
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: IntoPy<PyObject>,
        E: Into<PyErr>,
    {
        let wrap = async move {
            let obj = future.await.map_err(Into::into)?;
            // SAFETY: GIL is acquired when future is polled (see `Coroutine::poll`)
            Ok(obj.into_py(unsafe { Python::assume_gil_acquired() }))
        };
        Self {
            name,
            qualname_prefix,
            future: Some(Box::pin(panic::AssertUnwindSafe(wrap).catch_unwind())),
            waker: None,
        }
    }

    fn poll(
        &mut self,
        py: Python<'_>,
        throw: Option<PyObject>,
    ) -> PyResult<IterNextOutput<PyObject, PyObject>> {
        // raise if the coroutine has already been run to completion
        let future_rs = match self.future {
            Some(ref mut fut) => fut,
            None => return Err(PyRuntimeError::new_err(COROUTINE_REUSED_ERROR)),
        };
        // reraise thrown exception it
        if let Some(exc) = throw {
            self.close();
            return Err(PyErr::from_value(exc.as_ref(py)));
        }
        // create a new waker, or try to reset it in place
        if let Some(waker) = self.waker.as_mut().and_then(Arc::get_mut) {
            waker.reset();
        } else {
            self.waker = Some(Arc::new(AsyncioWaker::new()));
        }
        let waker = futures_util::task::waker(self.waker.clone().unwrap());
        // poll the Rust future and forward its results if ready
        if let Poll::Ready(res) = future_rs.as_mut().poll(&mut Context::from_waker(&waker)) {
            self.close();
            return match res {
                Ok(res) => Ok(IterNextOutput::Return(res?)),
                Err(err) => Err(PanicException::from_panic_payload(err)),
            };
        }
        // otherwise, initialize the waker `asyncio.Future`
        if let Some(future) = self.waker.as_ref().unwrap().initialize_future(py)? {
            // `asyncio.Future` must be awaited; fortunately, it implements `__iter__ = __await__`
            // and will yield itself if its result has not been set in polling above
            if let Some(future) = PyIterator::from_object(future).unwrap().next() {
                // future has not been leaked into Python for now, and Rust code can only call
                // `set_result(None)` in `ArcWake` implementation, so it's safe to unwrap
                return Ok(IterNextOutput::Yield(future.unwrap().into()));
            }
        }
        // if waker has been waken during future polling, this is roughly equivalent to
        // `await asyncio.sleep(0)`, so just yield `None`.
        Ok(IterNextOutput::Yield(py.None().into()))
    }
}

pub(crate) fn iter_result(result: IterNextOutput<PyObject, PyObject>) -> PyResult<PyObject> {
    match result {
        IterNextOutput::Yield(ob) => Ok(ob),
        IterNextOutput::Return(ob) => Err(PyStopIteration::new_err(ob)),
    }
}

#[pymethods(crate = "crate")]
impl Coroutine {
    #[getter]
    fn __name__(&self, py: Python<'_>) -> PyResult<Py<PyString>> {
        match &self.name {
            Some(name) => Ok(name.clone_ref(py)),
            None => Err(PyAttributeError::new_err("__name__")),
        }
    }

    #[getter]
    fn __qualname__(&self, py: Python<'_>) -> PyResult<Py<PyString>> {
        match (&self.name, &self.qualname_prefix) {
            (Some(name), Some(prefix)) => Ok(format!("{}.{}", prefix, name.as_ref(py).to_str()?)
                .as_str()
                .into_py(py)),
            (Some(name), None) => Ok(name.clone_ref(py)),
            (None, _) => Err(PyAttributeError::new_err("__qualname__")),
        }
    }

    fn send(&mut self, py: Python<'_>, _value: &PyAny) -> PyResult<PyObject> {
        iter_result(self.poll(py, None)?)
    }

    fn throw(&mut self, py: Python<'_>, exc: PyObject) -> PyResult<PyObject> {
        iter_result(self.poll(py, Some(exc))?)
    }

    fn close(&mut self) {
        // the Rust future is dropped, and the field set to `None`
        // to indicate the coroutine has been run to completion
        drop(self.future.take());
    }

    fn __await__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<IterNextOutput<PyObject, PyObject>> {
        self.poll(py, None)
    }
}
