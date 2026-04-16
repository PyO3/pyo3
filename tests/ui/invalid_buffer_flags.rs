use pyo3::buffer::{PyBufferRequest, PyUntypedBufferView};
use pyo3::prelude::*;
use pyo3::types::PyBytes;

fn main() {
    Python::attach(|py| {
        let bytes = PyBytes::new(py, &[1, 2, 3]);
        PyUntypedBufferView::with_flags(&bytes, PyBufferRequest::strided(), |view| {
            view.format();
        })
        .unwrap();
    });
}
