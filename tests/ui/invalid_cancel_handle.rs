use pyo3::prelude::*;

#[pyfunction]
async fn cancel_handle_repeated(#[pyo3(cancel_handle, cancel_handle)] _param: String) {}

#[pyfunction]
async fn cancel_handle_repeated2(
    #[pyo3(cancel_handle)] _param: String,
    #[pyo3(cancel_handle)] _param2: String,
) {
}

#[pyfunction]
fn cancel_handle_synchronous(#[pyo3(cancel_handle)] _param: String) {}

#[pyfunction]
async fn cancel_handle_wrong_type(#[pyo3(cancel_handle)] _param: String) {}

#[pyfunction]
async fn missing_cancel_handle_attribute(_param: pyo3::coroutine::CancelHandle) {}

#[pyfunction]
async fn cancel_handle_and_from_py_with(
    #[pyo3(cancel_handle, from_py_with = "cancel_handle")] _param: pyo3::coroutine::CancelHandle,
) {
}

fn main() {}
