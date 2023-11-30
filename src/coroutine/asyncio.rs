//! Coroutine implementation compatible with asyncio.
use pyo3_macros::pyfunction;

use crate::{
    intern,
    sync::GILOnceCell,
    types::{PyAnyMethods, PyCFunction, PyIterator},
    wrap_pyfunction_bound, Bound, IntoPy, Py, PyAny, PyObject, PyResult, Python,
};

/// `asyncio.get_running_loop`
fn get_running_loop(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    static GET_RUNNING_LOOP: GILOnceCell<PyObject> = GILOnceCell::new();
    let import = || -> PyResult<_> {
        let module = py.import_bound("asyncio")?;
        Ok(module.getattr("get_running_loop")?.into())
    };
    GET_RUNNING_LOOP
        .get_or_try_init(py, import)?
        .bind(py)
        .call0()
}

/// Asyncio-compatible coroutine waker.
///
/// Polling a Rust future yields an `asyncio.Future`, whose `set_result` method is called
/// when `Waker::wake` is called.
pub(super) struct AsyncioWaker {
    event_loop: PyObject,
    future: PyObject,
}

impl AsyncioWaker {
    pub(super) fn new(py: Python<'_>) -> PyResult<Self> {
        let event_loop = get_running_loop(py)?.into_py(py);
        let future = event_loop.call_method0(py, "create_future")?;
        Ok(Self { event_loop, future })
    }

    pub(super) fn yield_(&self, py: Python<'_>) -> PyResult<PyObject> {
        let __await__;
        // `asyncio.Future` must be awaited; in normal case, it implements  `__iter__ = __await__`,
        // but `create_future` may have been overriden
        let mut iter = match PyIterator::from_bound_object(self.future.bind(py)) {
            Ok(iter) => iter,
            Err(_) => {
                __await__ = self.future.call_method0(py, intern!(py, "__await__"))?;
                PyIterator::from_bound_object(__await__.bind(py))?
            }
        };
        // future has not been wakened (because `yield_waken` would have been called
        // otherwise), so it is expected to yield itself
        Ok(iter.next().expect("future didn't yield")?.into_py(py))
    }

    #[allow(clippy::unnecessary_wraps)]
    pub(super) fn yield_waken(py: Python<'_>) -> PyResult<PyObject> {
        Ok(py.None())
    }

    pub(super) fn wake(&self, py: Python<'_>) -> PyResult<()> {
        static RELEASE_WAITER: GILOnceCell<Py<PyCFunction>> = GILOnceCell::new();
        let release_waiter = RELEASE_WAITER.get_or_try_init(py, || {
            wrap_pyfunction_bound!(release_waiter, py).map(Into::into)
        })?;
        // `Future.set_result` must be called in event loop thread,
        // so it requires `call_soon_threadsafe`
        let call_soon_threadsafe = self.event_loop.call_method1(
            py,
            intern!(py, "call_soon_threadsafe"),
            (release_waiter, &self.future),
        );
        if let Err(err) = call_soon_threadsafe {
            // `call_soon_threadsafe` will raise if the event loop is closed;
            // instead of catching an unspecific `RuntimeError`, check directly if it's closed.
            let is_closed = self.event_loop.call_method0(py, "is_closed")?;
            if !is_closed.extract(py)? {
                return Err(err);
            }
        }
        Ok(())
    }
}

/// Call `future.set_result` if the future is not done.
///
/// Future can be cancelled by the event loop before being wakened.
/// See <https://github.com/python/cpython/blob/main/Lib/asyncio/tasks.py#L452C5-L452C5>
#[pyfunction(crate = "crate")]
fn release_waiter(future: &Bound<'_, PyAny>) -> PyResult<()> {
    let done = future.call_method0(intern!(future.py(), "done"))?;
    if !done.extract::<bool>()? {
        future.call_method1(intern!(future.py(), "set_result"), (future.py().None(),))?;
    }
    Ok(())
}
