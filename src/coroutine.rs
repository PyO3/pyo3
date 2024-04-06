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
    coroutine::{cancel::ThrowCallback, waker::AsyncioWaker},
    exceptions::{PyAttributeError, PyRuntimeError, PyStopIteration},
    panic::PanicException,
    types::{string::PyStringMethods, PyIterator, PyString},
    Bound, IntoPy, Py, PyAny, PyErr, PyObject, PyResult, Python,
};

pub(crate) mod cancel;
mod waker;

use crate::marker::Ungil;
pub use cancel::CancelHandle;

const COROUTINE_REUSED_ERROR: &str = "cannot reuse already awaited coroutine";

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

    fn poll(&mut self, py: Python<'_>, throw: Option<PyObject>) -> PyResult<PyObject> {
        // raise if the coroutine has already been run to completion
        let future_rs = match self.future {
            Some(ref mut fut) => fut,
            None => return Err(PyRuntimeError::new_err(COROUTINE_REUSED_ERROR)),
        };
        // reraise thrown exception it
        match (throw, &self.throw_callback) {
            (Some(exc), Some(cb)) => cb.throw(exc),
            (Some(exc), None) => {
                self.close();
                return Err(PyErr::from_value_bound(exc.into_bound(py)));
            }
            (None, _) => {}
        }
        // create a new waker, or try to reset it in place
        if let Some(waker) = self.waker.as_mut().and_then(Arc::get_mut) {
            waker.reset();
        } else {
            self.waker = Some(Arc::new(AsyncioWaker::new()));
        }
        // poll the future and forward its results if ready
        // polling is UnwindSafe because the future is dropped in case of panic
        let waker = Waker::from(self.waker.clone().unwrap());
        let poll = || future_rs.as_mut().poll(py, &waker);
        match panic::catch_unwind(panic::AssertUnwindSafe(poll)) {
            Ok(Poll::Ready(res)) => {
                self.close();
                return Err(PyStopIteration::new_err(res?));
            }
            Err(err) => {
                self.close();
                return Err(PanicException::from_panic_payload(err));
            }
            _ => {}
        }
        // otherwise, initialize the waker `asyncio.Future`
        if let Some(future) = self.waker.as_ref().unwrap().initialize_future(py)? {
            // `asyncio.Future` must be awaited; fortunately, it implements `__iter__ = __await__`
            // and will yield itself if its result has not been set in polling above
            if let Some(future) = PyIterator::from_bound_object(&future.as_borrowed())
                .unwrap()
                .next()
            {
                // future has not been leaked into Python for now, and Rust code can only call
                // `set_result(None)` in `Wake` implementation, so it's safe to unwrap
                return Ok(future.unwrap().into());
            }
        }
        // if waker has been waken during future polling, this is roughly equivalent to
        // `await asyncio.sleep(0)`, so just yield `None`.
        Ok(py.None().into_py(py))
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

    fn send(&mut self, py: Python<'_>, _value: &Bound<'_, PyAny>) -> PyResult<PyObject> {
        self.poll(py, None)
    }

    fn throw(&mut self, py: Python<'_>, exc: PyObject) -> PyResult<PyObject> {
        self.poll(py, Some(exc))
    }

    fn close(&mut self) {
        // the Rust future is dropped, and the field set to `None`
        // to indicate the coroutine has been run to completion
        drop(self.future.take());
    }

    fn __await__(self_: Py<Self>) -> Py<Self> {
        self_
    }

    fn __next__(&mut self, py: Python<'_>) -> PyResult<PyObject> {
        self.poll(py, None)
    }
}
