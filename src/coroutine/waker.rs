use crate::sync::PyOnceLock;
use crate::types::any::PyAnyMethods;
use crate::types::PyCFunction;
use crate::{intern, wrap_pyfunction, Bound, Py, PyAny, PyResult, Python};
use pyo3_macros::pyfunction;
use std::sync::Arc;
use std::task::Wake;

/// Lazy `asyncio.Future` wrapper, implementing [`Wake`] by calling `Future.set_result`.
///
/// asyncio future is let uninitialized until [`initialize_future`][1] is called.
/// If [`wake`][2] is called before future initialization (during Rust future polling),
/// [`initialize_future`][1] will return `None` (it is roughly equivalent to `asyncio.sleep(0)`)
///
/// [1]: AsyncioWaker::initialize_future
/// [2]: AsyncioWaker::wake
pub struct AsyncioWaker(PyOnceLock<Option<LoopAndFuture>>);

impl AsyncioWaker {
    pub(super) fn new() -> Self {
        Self(PyOnceLock::new())
    }

    pub(super) fn reset(&mut self) {
        self.0.take();
    }

    pub(super) fn initialize_future<'py>(
        &self,
        py: Python<'py>,
    ) -> PyResult<Option<&Bound<'py, PyAny>>> {
        let init = || LoopAndFuture::new(py).map(Some);
        let loop_and_future = self.0.get_or_try_init(py, init)?.as_ref();
        Ok(loop_and_future.map(|LoopAndFuture { future, .. }| future.bind(py)))
    }
}

impl Wake for AsyncioWaker {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        Python::attach(|py| {
            if let Some(loop_and_future) = self.0.get_or_init(py, || None) {
                loop_and_future
                    .set_result(py)
                    .expect("unexpected error in coroutine waker");
            }
        });
    }
}

struct LoopAndFuture {
    event_loop: Py<PyAny>,
    future: Py<PyAny>,
}

impl LoopAndFuture {
    fn new(py: Python<'_>) -> PyResult<Self> {
        static GET_RUNNING_LOOP: PyOnceLock<Py<PyAny>> = PyOnceLock::new();
        let import = || -> PyResult<_> {
            let module = py.import("asyncio")?;
            Ok(module.getattr("get_running_loop")?.into())
        };
        let event_loop = GET_RUNNING_LOOP.get_or_try_init(py, import)?.call0(py)?;
        let future = event_loop.call_method0(py, "create_future")?;
        Ok(Self { event_loop, future })
    }

    fn set_result(&self, py: Python<'_>) -> PyResult<()> {
        static RELEASE_WAITER: PyOnceLock<Py<PyCFunction>> = PyOnceLock::new();
        let release_waiter = RELEASE_WAITER.get_or_try_init(py, || {
            wrap_pyfunction!(release_waiter, py).map(Bound::unbind)
        })?;
        // `Future.set_result` must be called in event loop thread,
        // so it requires `call_soon_threadsafe`
        let call_soon_threadsafe = self.event_loop.call_method1(
            py,
            intern!(py, "call_soon_threadsafe"),
            (release_waiter, self.future.bind(py)),
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
/// Future can be cancelled by the event loop before being waken.
/// See <https://github.com/python/cpython/blob/main/Lib/asyncio/tasks.py#L452C5-L452C5>
#[pyfunction(crate = "crate")]
fn release_waiter(future: &Bound<'_, PyAny>) -> PyResult<()> {
    let done = future.call_method0(intern!(future.py(), "done"))?;
    if !done.extract::<bool>()? {
        future.call_method1(intern!(future.py(), "set_result"), (future.py().None(),))?;
    }
    Ok(())
}
