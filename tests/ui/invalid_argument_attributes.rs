use pyo3::prelude::*;

#[pyfunction]
fn invalid_attribute(#[pyo3(get)] param: String) {}

#[pyfunction]
fn from_py_with_no_value(#[pyo3(from_py_with)] param: String) {}

#[pyfunction]
fn from_py_with_string(#[pyo3("from_py_with")] param: String) {}

#[pyfunction]
fn from_py_with_value_not_a_string(#[pyo3(from_py_with = func)] param: String) {}

#[pyfunction]
fn from_py_with_repeated(#[pyo3(from_py_with = "func", from_py_with = "func")] param: String) {}

#[pyfunction]
async fn from_py_with_value_and_cancel_handle(
    #[pyo3(from_py_with = "func", cancel_handle)] _param: String,
) {
}

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

fn main() {}
