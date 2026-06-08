use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};

fn main() {
    // Closure must be `Sync`
    Python::attach(|py| {
        let data = std::cell::Cell::new(0);
        let closure_fn = move |_args: &Bound<'_, PyTuple>,
                               _kwargs: Option<&Bound<'_, PyDict>>|
              -> PyResult<()> {
            let _ = data.clone();
            Ok(())
        };

        PyCFunction::new_closure(py, None, None, closure_fn).unwrap();
        //~^ ERROR: `Cell<i32>` cannot be shared between threads safely
    });
}
