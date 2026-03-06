use pyo3::prelude::*;

#[pyfunction]
async fn cancel_handle_repeated(#[pyo3(cancel_handle, cancel_handle)] _param: String) {}
//~^ ERROR: `cancel_handle` may only be specified once per argument

#[pyfunction]
async fn cancel_handle_repeated2(
//~^ ERROR: async functions are only supported with the `experimental-async` feature
    #[pyo3(cancel_handle)] _param: String,
    #[pyo3(cancel_handle)] _param2: String,
) {
}

#[pyfunction]
fn cancel_handle_synchronous(#[pyo3(cancel_handle)] _param: String) {}
//~^ ERROR: `cancel_handle` attribute can only be used with `async fn`

#[pyfunction]
async fn cancel_handle_wrong_type(#[pyo3(cancel_handle)] _param: String) {}
//~^ ERROR: async functions are only supported with the `experimental-async` feature

#[pyfunction]
async fn missing_cancel_handle_attribute(_param: pyo3::coroutine::CancelHandle) {}
//~^ ERROR: failed to resolve: could not find `coroutine` in `pyo3`
//~| ERROR: async functions are only supported with the `experimental-async` feature

#[pyfunction]
async fn cancel_handle_and_from_py_with(
    #[pyo3(cancel_handle, from_py_with = cancel_handle)] _param: pyo3::coroutine::CancelHandle,
//~^ ERROR: failed to resolve: could not find `coroutine` in `pyo3`
//~| ERROR: `from_py_with` and `cancel_handle` cannot be specified together
) {
}

fn main() {}
