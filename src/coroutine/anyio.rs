//! Coroutine implementation using sniffio to select the appropriate implementation,
//! compatible with anyio.
use crate::{
    coroutine::{asyncio::AsyncioWaker, trio::TrioWaker},
    exceptions::PyRuntimeError,
    sync::GILOnceCell,
    types::PyAnyMethods,
    PyObject, PyResult, Python,
};

enum AsyncLib {
    Asyncio,
    Trio,
}

fn current_async_library(py: Python<'_>) -> PyResult<AsyncLib> {
    static CURRENT_ASYNC_LIBRARY: GILOnceCell<Option<PyObject>> = GILOnceCell::new();
    let import = || -> PyResult<_> {
        Ok(match py.import("sniffio") {
            Ok(module) => Some(module.getattr("current_async_library")?.into()),
            Err(_) => None,
        })
    };
    let Some(func) = CURRENT_ASYNC_LIBRARY.get_or_try_init(py, import)? else {
        return Ok(AsyncLib::Asyncio);
    };
    match func.bind(py).call0()?.extract()? {
        "asyncio" => Ok(AsyncLib::Asyncio),
        "trio" => Ok(AsyncLib::Trio),
        rt => Err(PyRuntimeError::new_err(format!("unsupported runtime {rt}"))),
    }
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
        match current_async_library(py)? {
            AsyncLib::Asyncio => Ok(Self::Asyncio(AsyncioWaker::new(py)?)),
            AsyncLib::Trio => Ok(Self::Trio(TrioWaker::new(py)?)),
        }
    }

    pub(super) fn yield_(&self, py: Python<'_>) -> PyResult<PyObject> {
        match self {
            AnyioWaker::Asyncio(w) => w.yield_(py),
            AnyioWaker::Trio(w) => w.yield_(py),
        }
    }

    pub(super) fn yield_waken(py: Python<'_>) -> PyResult<PyObject> {
        match current_async_library(py)? {
            AsyncLib::Asyncio => AsyncioWaker::yield_waken(py),
            AsyncLib::Trio => TrioWaker::yield_waken(py),
        }
    }

    pub(super) fn wake(&self, py: Python<'_>) -> PyResult<()> {
        match self {
            AnyioWaker::Asyncio(w) => w.wake(py),
            AnyioWaker::Trio(w) => w.wake(py),
        }
    }
}
