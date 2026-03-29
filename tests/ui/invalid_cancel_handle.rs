use pyo3::prelude::*;

#[pyfunction]
async fn cancel_handle_repeated(#[pyo3(cancel_handle, cancel_handle)] _param: String) {}
//~^ ERROR: `cancel_handle` may only be specified once per argument

#[pyfunction]
async fn cancel_handle_repeated2(
    #[pyo3(cancel_handle)] _param: String,
    #[pyo3(cancel_handle)] _param2: String,
    //~^ ERROR: `cancel_handle` may only be specified once
) {
}

#[pyfunction]
fn cancel_handle_synchronous(#[pyo3(cancel_handle)] _param: String) {}
//~^ ERROR: `cancel_handle` attribute can only be used with `async fn`

#[pyfunction]
//~^ ERROR: mismatched types
async fn cancel_handle_wrong_type(#[pyo3(cancel_handle)] _param: String) {}

#[pyfunction]
async fn missing_cancel_handle_attribute(_param: pyo3::coroutine::CancelHandle) {}
//~^ ERROR: `CancelHandle` cannot be used as a Python function argument
//~| ERROR: `CancelHandle` cannot be used as a Python function argument
//~| ERROR: `CancelHandle` cannot be used as a Python function argument

#[pyfunction]
async fn cancel_handle_and_from_py_with(
    #[pyo3(cancel_handle, from_py_with = cancel_handle)] _param: pyo3::coroutine::CancelHandle,
    //~^ ERROR: `from_py_with` and `cancel_handle` cannot be specified together
) {
}

fn main() {}
