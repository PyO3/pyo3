use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::sync::atomic::{AtomicU64, Ordering};

/// A function decorator that keeps track how often it is called.
///
/// It otherwise doesn't do anything special.
#[pyclass(name = "Counter")]
pub struct PyCounter {
    // Keeps track of how many calls have gone through.
    //
    // See the discussion at the end for why an atomic is used.
    count: AtomicU64,

    // This is the actual function being wrapped.
    wraps: Py<PyAny>,
}

#[pymethods]
impl PyCounter {
    // Note that we don't validate whether `wraps` is actually callable.
    //
    // While we could use `PyAny::is_callable` for that, it has some flaws:
    //    1. It doesn't guarantee the object can actually be called successfully
    //    2. We still need to handle any exceptions that the function might raise
    #[new]
    fn __new__(wraps: Py<PyAny>) -> Self {
        PyCounter {
            count: AtomicU64::new(0),
            wraps,
        }
    }

    #[getter]
    fn count(&self) -> u64 {
        self.count.load(Ordering::Relaxed)
    }

    #[args(args = "*", kwargs = "**")]
    fn __call__(
        &self,
        py: Python<'_>,
        args: &PyTuple,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Py<PyAny>> {
        let old_count = self.count.fetch_add(1, Ordering::Relaxed);
        let name = self.wraps.getattr(py, "__name__")?;

        println!("{} has been called {} time(s).", name, old_count + 1);

        // After doing something, we finally forward the call to the wrapped function
        let ret = self.wraps.call(py, args, kwargs)?;

        // We could do something with the return value of
        // the function before returning it
        Ok(ret)
    }
}

#[pymodule]
pub fn decorator(_py: Python<'_>, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyCounter>()?;
    Ok(())
}
