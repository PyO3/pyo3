use pyo3::prelude::*;

fn test_not_send_detach(py: Python<'_>) {
    py.detach(|| { drop(py); });
}

fn main() {
    Python::attach(|py| {
        test_not_send_detach(py);
    })
}
