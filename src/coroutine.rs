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
    exceptions::{PyAttributeError, PyGeneratorExit, PyRuntimeError, PyStopIteration},
    panic::PanicException,
    types::{string::PyStringMethods, PyString},
    Bound, IntoPyObject, IntoPyObjectExt, Py, PyErr, PyObject, PyResult, Python,
};

#[cfg(feature = "anyio")]
mod anyio;
mod asyncio;
mod awaitable;
mod cancel;
#[cfg(feature = "anyio")]
mod trio;
mod waker;

pub use awaitable::await_in_coroutine;
pub use cancel::{CancelHandle, ThrowCallback};

const COROUTINE_REUSED_ERROR: &str = "cannot reuse already awaited coroutine";

pub(crate) enum CoroOp {
    Send(PyObject),
    Throw(PyObject),
    Close,
}

/// Python coroutine wrapping a [`Future`].
#[pyclass(crate = "crate")]
pub struct Coroutine {
    name: Option<Py<PyString>>,
    qualname_prefix: Option<&'static str>,
    throw_callback: Option<ThrowCallback>,
    future: Option<Pin<Box<dyn Future<Output = PyResult<PyObject>> + Send>>>,
    waker: Option<Arc<CoroutineWaker>>,
}

// Safety: `Coroutine` is allowed to be `Sync` even though the future is not,
// because the future is polled with `&mut self` receiver
unsafe impl Sync for Coroutine {}

impl Coroutine {
    ///  Wrap a future into a Python coroutine.
    ///
    /// Coroutine `send` polls the wrapped future, ignoring the value passed
    /// (should always be `None` anyway).
    ///
    /// `Coroutine `throw` drop the wrapped future and reraise the exception passed
    pub(crate) fn new<'py, F, T, E>(
        name: Option<Bound<'py, PyString>>,
        qualname_prefix: Option<&'static str>,
        throw_callback: Option<ThrowCallback>,
        future: F,
    ) -> Self
    where
        F: Future<Output = Result<T, E>> + Send + 'static,
        T: IntoPyObject<'py>,
        E: Into<PyErr>,
    {
        let wrap = async move {
            let obj = future.await.map_err(Into::into)?;
            // SAFETY: GIL is acquired when future is polled (see `Coroutine::poll`)
            obj.into_py_any(unsafe { Python::assume_gil_acquired() })
        };
        Self {
            name: name.map(Bound::unbind),
            qualname_prefix,
            throw_callback,
            future: Some(Box::pin(wrap)),
            waker: None,
        }
    }

    fn poll_inner(&mut self, py: Python<'_>, mut op: CoroOp) -> PyResult<PyObject> {
        // raise if the coroutine has already been run to completion
        let future_rs = match self.future {
            Some(ref mut fut) => fut,
            None => return Err(PyRuntimeError::new_err(COROUTINE_REUSED_ERROR)),
        };
        // if the future is not pending on a Python awaitable,
        // execute throw callback or complete on close
        if !matches!(self.waker, Some(ref w) if w.is_delegated(py)) {
            match op {
                send @ CoroOp::Send(_) => op = send,
                CoroOp::Throw(exc) => match &self.throw_callback {
                    Some(cb) => {
                        cb.throw(exc.clone_ref(py));
                        op = CoroOp::Send(py.None());
                    }
                    None => return Err(PyErr::from_value(exc.into_bound(py))),
                },
                CoroOp::Close => return Err(PyGeneratorExit::new_err(py.None())),
            };
        }
        // create a new waker, or try to reset it in place
        if let Some(waker) = self.waker.as_mut().and_then(Arc::get_mut) {
            waker.reset(op);
        } else {
            self.waker = Some(Arc::new(CoroutineWaker::new(op)));
        }
        // poll the future and forward its results if ready; otherwise, yield from waker
        // polling is UnwindSafe because the future is dropped in case of panic
        let waker = Waker::from(self.waker.clone().unwrap());
        let poll = || future_rs.as_mut().poll(&mut Context::from_waker(&waker));
        match panic::catch_unwind(panic::AssertUnwindSafe(poll)) {
            Err(err) => Err(PanicException::from_panic_payload(err)),
            // See #4407, `PyStopIteration::new_err` argument must be wrap in a tuple,
            // otherwise, when a tuple is returned, its fields would be expanded as error
            // arguments
            Ok(Poll::Ready(res)) => Err(PyStopIteration::new_err((res?,))),
            Ok(Poll::Pending) => match self.waker.as_ref().unwrap().yield_(py) {
                Ok(to_yield) => Ok(to_yield),
                Err(err) => Err(err),
            },
        }
    }

    fn poll(&mut self, py: Python<'_>, op: CoroOp) -> PyResult<PyObject> {
        let result = self.poll_inner(py, op);
        if result.is_err() {
            // the Rust future is dropped, and the field set to `None`
            // to indicate the coroutine has been run to completion
            drop(self.future.take());
        }
        result
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
    fn __qualname__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyString>> {
        match (&self.name, &self.qualname_prefix) {
            (Some(name), Some(prefix)) => Ok(PyString::new(
                py,
                &format!("{}.{}", prefix, name.bind(py).to_cow()?),
            )),
            (Some(name), None) => Ok(name.bind(py).clone()),
            (None, _) => Err(PyAttributeError::new_err("__qualname__")),
        }
    }

    fn send(&mut self, py: Python<'_>, value: PyObject) -> PyResult<PyObject> {
        self.poll(py, CoroOp::Send(value))
    }

    fn throw(&mut self, py: Python<'_>, exc: PyObject) -> PyResult<PyObject> {
        self.poll(py, CoroOp::Throw(exc))
    }

    fn close(&mut self, py: Python<'_>) -> PyResult<()> {
        match self.poll(py, CoroOp::Close) {
            Ok(_) => Ok(()),
            Err(err) if err.is_instance_of::<PyGeneratorExit>(py) => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn __await__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        self.poll(py, CoroOp::Send(py.None()))
    }
}
