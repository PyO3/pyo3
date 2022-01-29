use pyo3::prelude::*;

fn test_not_send_allow_threads(py: Python) {
    py.allow_threads(|| { drop(py); });
}

fn main() {
    Python::with_gil(|py| {
        test_not_send_allow_threads(py);
    })
}
