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
    marker::Ungil,
    panic::PanicException,
    types::{string::PyStringMethods, PyString},
    IntoPy, Py, PyErr, PyObject, PyResult, Python,
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

trait CoroutineFuture: Send {
    fn poll(self: Pin<&mut Self>, py: Python<'_>, waker: &Waker) -> Poll<PyResult<PyObject>>;
}

impl<F, T, E> CoroutineFuture for F
where
    F: Future<Output = Result<T, E>> + Send,
    T: IntoPy<PyObject> + Send,
    E: Into<PyErr> + Send,
{
    fn poll(self: Pin<&mut Self>, py: Python<'_>, waker: &Waker) -> Poll<PyResult<PyObject>> {
        self.poll(&mut Context::from_waker(waker))
            .map_ok(|obj| obj.into_py(py))
            .map_err(Into::into)
    }
}

struct AllowThreads<F> {
    future: F,
}

impl<F, T, E> CoroutineFuture for AllowThreads<F>
where
    F: Future<Output = Result<T, E>> + Send + Ungil,
    T: IntoPy<PyObject> + Send + Ungil,
    E: Into<PyErr> + Send + Ungil,
{
    fn poll(self: Pin<&mut Self>, py: Python<'_>, waker: &Waker) -> Poll<PyResult<PyObject>> {
        // SAFETY: future field is pinned when self is
        let future = unsafe { self.map_unchecked_mut(|a| &mut a.future) };
        py.allow_threads(|| future.poll(&mut Context::from_waker(waker)))
            .map_ok(|obj| obj.into_py(py))
            .map_err(Into::into)
    }
}

/// Python coroutine wrapping a [`Future`].
#[pyclass(crate = "crate")]
pub struct Coroutine {
    name: Option<Py<PyString>>,
    qualname_prefix: Option<&'static str>,
    throw_callback: Option<ThrowCallback>,
    future: Option<Pin<Box<dyn CoroutineFuture>>>,
    waker: Option<Arc<CoroutineWaker>>,
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
        throw_callback: Option<ThrowCallback>,
        allow_threads: bool,
        future: F,
    ) -> Self
    where
        F: Future<Output = Result<T, E>> + Send + Ungil + 'static,
        T: IntoPy<PyObject> + Send + Ungil,
        E: Into<PyErr> + Send + Ungil,
    {
        Self {
            name,
            qualname_prefix,
            throw_callback,
            future: Some(if allow_threads {
                Box::pin(AllowThreads { future })
            } else {
                Box::pin(future)
            }),
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
                    None => return Err(PyErr::from_value_bound(exc.into_bound(py))),
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
        let poll = || future_rs.as_mut().poll(py, &waker);
        match panic::catch_unwind(panic::AssertUnwindSafe(poll)) {
            Err(err) => Err(PanicException::from_panic_payload(err)),
            Ok(Poll::Ready(res)) => Err(PyStopIteration::new_err(res?)),
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
    fn __qualname__(&self, py: Python<'_>) -> PyResult<Py<PyString>> {
        match (&self.name, &self.qualname_prefix) {
            (Some(name), Some(prefix)) => Ok(format!("{}.{}", prefix, name.bind(py).to_cow()?)
                .as_str()
                .into_py(py)),
            (Some(name), None) => Ok(name.clone_ref(py)),
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
