//! Coroutine implementation using sniffio to select the appropriate implementation,
//! compatible with anyio.
use crate::{
    coroutine::{asyncio::AsyncioWaker, trio::TrioWaker},
    exceptions::PyRuntimeError,
    sync::GILOnceCell,
    types::PyAnyMethods,
    Bound, PyAny, PyErr, PyObject, PyResult, Python,
};

fn current_async_library(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    static CURRENT_ASYNC_LIBRARY: GILOnceCell<PyObject> = GILOnceCell::new();
    let import = || -> PyResult<_> {
        let module = py.import_bound("sniffio")?;
        Ok(module.getattr("current_async_library")?.into())
    };
    CURRENT_ASYNC_LIBRARY
        .get_or_try_init(py, import)?
        .bind(py)
        .call0()
}

fn unsupported(runtime: &str) -> PyErr {
    PyRuntimeError::new_err(format!("unsupported runtime {rt}", rt = runtime))
}

/// Sniffio/anyio-compatible coroutine waker.
///
/// Polling a Rust future calls `sniffio.current_async_library` to select the appropriate
/// implementation, either asyncio or trio.
pub(super) enum AnyioWaker {
    /// [`AsyncioWaker`]
    Asyncio(AsyncioWaker),
    /// [`TrioWaker`]
    Trio(TrioWaker),
}

impl AnyioWaker {
    pub(super) fn new(py: Python<'_>) -> PyResult<Self> {
        let sniffed = current_async_library(py)?;
        match sniffed.extract()? {
            "asyncio" => Ok(Self::Asyncio(AsyncioWaker::new(py)?)),
            "trio" => Ok(Self::Trio(TrioWaker::new(py)?)),
            rt => Err(unsupported(rt)),
        }
    }

    pub(super) fn yield_(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self {
            AnyioWaker::Asyncio(w) => w.yield_(py),
            AnyioWaker::Trio(w) => w.yield_(py),
        }
    }

    pub(super) fn yield_waken(py: Python<'_>) -> PyResult<PyObject> {
        let sniffed = current_async_library(py)?;
        match sniffed.extract()? {
            "asyncio" => AsyncioWaker::yield_waken(py),
            "trio" => TrioWaker::yield_waken(py),
            rt => Err(unsupported(rt)),
        }
    }

    pub(super) fn wake(&self, py: Python<'_>) -> PyResult<()> {
        match self {
            AnyioWaker::Asyncio(w) => w.wake(py),
            AnyioWaker::Trio(w) => w.wake(py),
        }
    }
}
