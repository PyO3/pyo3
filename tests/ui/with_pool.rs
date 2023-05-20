use pyo3::prelude::*;

fn reuse_old_token_in_with_pool(old_py: Python<'_>) {
    old_py.with_pool(|new_py| { drop(old_py); });
}

fn main() {
    Python::with_gil(|py| {
        reuse_old_token_in_with_pool(py);
    })
}
