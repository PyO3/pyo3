//! Python coroutine implementation, used notably when wrapping `async fn`
//! with `#[pyfunction]`/`#[pymethods]`.
use std::{
    future::Future,
    panic,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use pyo3_macros::{pyclass, pymethods};

use crate::{
    coroutine::waker::CoroutineWaker,
    exceptions::{PyRuntimeError, PyStopIteration},
    panic::PanicException,
    pyclass::IterNextOutput,
    types::PyString,
    IntoPy, Py, PyErr, PyObject, PyResult, Python,
};

#[cfg(feature = "anyio")]
mod anyio;
mod asyncio;
pub(crate) mod cancel;
#[cfg(feature = "anyio")]
mod trio;
pub(crate) mod waker;

pub use cancel::{CancelHandle, ThrowCallback};

const COROUTINE_REUSED_ERROR: &str = "cannot reuse already awaited coroutine";

trait CoroutineFuture {
    fn poll(
        self: Pin<&mut Self>,
        py: Python<'_>,
        waker: &Waker,
        allow_threads: bool,
    ) -> Poll<PyResult<PyObject>>;
}

impl<F, T, E> CoroutineFuture for F
where
    F: Future<Output = Result<T, E>> + Send,
    T: IntoPy<PyObject> + Send,
    E: Into<PyErr> + Send,
{
    fn poll(
        self: Pin<&mut Self>,
        py: Python<'_>,
        waker: &Waker,
        allow_threads: bool,
    ) -> Poll<PyResult<PyObject>> {
        let result = if allow_threads {
            py.allow_threads(|| self.poll(&mut Context::from_waker(waker)))
        } else {
            self.poll(&mut Context::from_waker(waker))
        };
        result.map_ok(|obj| obj.into_py(py)).map_err(Into::into)
    }
}

/// Python coroutine wrapping a [`Future`].
#[pyclass(crate = "crate")]
pub struct Coroutine {
    future: Option<Pin<Box<dyn CoroutineFuture + Send>>>,
    name: Py<PyString>,
    qualname_prefix: Option<&'static str>,
    throw_callback: Option<ThrowCallback>,
    allow_threads: bool,
    waker: Option<Arc<CoroutineWaker>>,
}

impl Coroutine {
    ///  Wrap a future into a Python coroutine.
    ///
    /// Coroutine `send` polls the wrapped future, ignoring the value passed
    /// (should always be `None` anyway).
    ///
    /// `Coroutine `throw` drop the wrapped future and reraise the exception passed.
    pub fn new<F, T, E>(name: impl Into<Py<PyString>>, future: F) -> Self
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: IntoPy<PyObject> + Send,
        E: Into<PyErr> + Send,
    {
        Self {
            future: Some(Box::pin(future)),
            name: name.into(),
            qualname_prefix: None,
            throw_callback: None,
            allow_threads: false,
            waker: None,
        }
    }

    /// Set a prefix for `__qualname__`, which will be joined with a "."
    pub fn with_qualname_prefix(mut self, prefix: impl Into<Option<&'static str>>) -> Self {
        self.qualname_prefix = prefix.into();
        self
    }

    /// Register a callback for coroutine `throw` method.
    ///
    /// The exception passed to `throw` is then redirected to this callback, notifying the
    /// associated [`CancelHandle`], without being reraised.
    pub fn with_throw_callback(mut self, callback: impl Into<Option<ThrowCallback>>) -> Self {
        self.throw_callback = callback.into();
        self
    }

    /// Release the GIL while polling the future wrapped.
    pub fn with_allow_threads(mut self, allow_threads: bool) -> Self {
        self.allow_threads = allow_threads;
        self
    }

    fn poll_inner(
        &mut self,
        py: Python<'_>,
        mut sent_result: Option<Result<PyObject, PyObject>>,
    ) -> PyResult<IterNextOutput<PyObject, PyObject>> {
        // raise if the coroutine has already been run to completion
        let future_rs = match self.future {
            Some(ref mut fut) => fut,
            None => return Err(PyRuntimeError::new_err(COROUTINE_REUSED_ERROR)),
        };
        // if the future is not pending on a Python awaitable,
        // execute throw callback or complete on close
        if !matches!(self.waker, Some(ref w) if w.is_delegated(py)) {
            match (sent_result, &self.throw_callback) {
                (res @ Some(Ok(_)), _) => sent_result = res,
                (Some(Err(err)), Some(cb)) => {
                    cb.throw(err.as_ref(py));
                    sent_result = Some(Ok(py.None().into()));
                }
                (Some(Err(err)), None) => return Err(PyErr::from_value(err.as_ref(py))),
                (None, _) => return Ok(IterNextOutput::Return(py.None().into())),
            }
        }
        // create a new waker, or try to reset it in place
        if let Some(waker) = self.waker.as_mut().and_then(Arc::get_mut) {
            waker.reset(sent_result);
        } else {
            self.waker = Some(Arc::new(CoroutineWaker::new(sent_result)));
        }
        // poll the future and forward its results if ready; otherwise, yield from waker
        // polling is UnwindSafe because the future is dropped in case of panic
        let waker = Waker::from(self.waker.clone().unwrap());
        let poll = || future_rs.as_mut().poll(py, &waker, self.allow_threads);
        match panic::catch_unwind(panic::AssertUnwindSafe(poll)) {
            Err(err) => Err(PanicException::from_panic_payload(err)),
            Ok(Poll::Ready(res)) => Ok(IterNextOutput::Return(res?)),
            Ok(Poll::Pending) => match self.waker.as_ref().unwrap().yield_(py) {
                Ok(to_yield) => Ok(IterNextOutput::Yield(to_yield)),
                Err(err) => Err(err),
            },
        }
    }

    fn poll(
        &mut self,
        py: Python<'_>,
        sent_result: Option<Result<PyObject, PyObject>>,
    ) -> PyResult<IterNextOutput<PyObject, PyObject>> {
        let result = self.poll_inner(py, sent_result);
        if matches!(result, Ok(IterNextOutput::Return(_)) | Err(_)) {
            // the Rust future is dropped, and the field set to `None`
            // to indicate the coroutine has been run to completion
            drop(self.future.take());
        }
        result
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
    fn __name__(&self, py: Python<'_>) -> Py<PyString> {
        self.name.clone_ref(py)
    }

    #[getter]
    fn __qualname__(&self, py: Python<'_>) -> PyResult<Py<PyString>> {
        Ok(match &self.qualname_prefix {
            Some(prefix) => format!("{}.{}", prefix, self.name.as_ref(py).to_str()?)
                .as_str()
                .into_py(py),
            None => self.name.clone_ref(py),
        })
    }

    fn send(&mut self, py: Python<'_>, value: PyObject) -> PyResult<PyObject> {
        iter_result(self.poll(py, Some(Ok(value)))?)
    }

    fn throw(&mut self, py: Python<'_>, exc: PyObject) -> PyResult<PyObject> {
        iter_result(self.poll(py, Some(Err(exc)))?)
    }

    fn close(&mut self, py: Python<'_>) -> PyResult<()> {
        self.poll(py, None)?;
        Ok(())
    }

    fn __await__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<IterNextOutput<PyObject, PyObject>> {
        self.poll(py, Some(Ok(py.None().into())))
    }
}
