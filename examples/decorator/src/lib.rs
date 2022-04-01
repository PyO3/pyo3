use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

/// A function decorator that keeps track how often it is called.
///
/// It otherwise doesn't do anything special.
#[pyclass(name = "Counter")]
pub struct PyCounter {
    // We use `#[pyo3(get)]` so that python can read the count but not mutate it.
    #[pyo3(get)]
    count: u64,

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
        PyCounter { count: 0, wraps }
    }

    #[args(args = "*", kwargs = "**")]
    fn __call__(
        &mut self,
        py: Python,
        args: &PyTuple,
        kwargs: Option<&PyDict>,
    ) -> PyResult<Py<PyAny>> {
        self.count += 1;
        let name = self.wraps.getattr(py, "__name__")?;

        println!("{} has been called {} time(s).", name, self.count);

        // After doing something, we finally forward the call to the wrapped function
        let ret = self.wraps.call(py, args, kwargs)?;

        // We could do something with the return value of
        // the function before returning it
        Ok(ret)
    }
}

#[pymodule]
pub fn decorator(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_class::<PyCounter>()?;
    Ok(())
}
