use pyo3::prelude::*;

fn test_not_send_allow_threads(py: Python<'_>) {
    py.allow_threads(|| { drop(py); });
}

fn main() {
    Python::attach(|py| {
        test_not_send_allow_threads(py);
    })
}
