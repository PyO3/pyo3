use pyo3::prelude::*;

#[pyfunction(allow_threads)]
async fn async_with_gil(_py: Python<'_>) {}

#[pyfunction(allow_threads)]
async fn async_with_bound(_obj: &Bound<'_, PyAny>) {}

#[pyfunction]
async fn cancel_handle_repeated(#[pyo3(cancel_handle, cancel_handle)] _param: i32) {}

#[pyfunction]
async fn cancel_handle_repeated2(
    #[pyo3(cancel_handle)] _param: i32,
    #[pyo3(cancel_handle)] _param2: i32,
) {
}

#[pyfunction]
fn cancel_handle_synchronous(#[pyo3(cancel_handle)] _param: i32) {}

#[pyfunction]
async fn cancel_handle_wrong_type(#[pyo3(cancel_handle)] _param: i32) {}

#[pyfunction]
async fn missing_cancel_handle_attribute(_param: pyo3::coroutine::CancelHandle) {}

#[pyfunction]
async fn cancel_handle_and_from_py_with(
    #[pyo3(cancel_handle, from_py_with = "cancel_handle")] _param: pyo3::coroutine::CancelHandle,
) {
}

fn main() {}
