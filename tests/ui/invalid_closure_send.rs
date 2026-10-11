use pyo3::prelude::*;
use pyo3::types::{PyCFunction, PyDict, PyTuple};

#[derive(Clone)]
struct NotSend(*mut std::ffi::c_void);
unsafe impl Sync for NotSend {}

fn main() {
    // Closure must be `Send`
    Python::attach(|py| {
        let value = NotSend(std::ptr::null_mut());
        let closure_fn = move |_args: &Bound<'_, PyTuple>,
                               _kwargs: Option<&Bound<'_, PyDict>>|
              -> PyResult<()> {
            let _ = value.clone();
            Ok(())
        };

        PyCFunction::new_closure(py, None, None, closure_fn).unwrap();
        //~^ ERROR: `*mut c_void` cannot be sent between threads safely
    });
}
