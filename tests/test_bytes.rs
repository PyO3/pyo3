use pyo3::prelude::*;
use pyo3::types::PyBytes;
use pyo3::wrap_pyfunction;

mod common;

#[pyfunction]
fn get_pybytes() -> PyObject {
    let gil = Python::acquire_gil();
    let py = gil.python();

    PyBytes::new(py, b"Hello World").into()
}

#[pyfunction]
fn bytes_pybytes_conversion(bytes: &[u8]) -> &[u8] {
    bytes
}

#[test]
fn test_get_pybytes() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let get_bytes = wrap_pyfunction!(get_pybytes)(py);
    py_assert!(py, get_bytes, "get_bytes() == b'Hello World'");
}

#[test]
fn test_pybytes_bytes_conversion() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let bytes_pybytes_conversion = wrap_pyfunction!(bytes_pybytes_conversion)(py);
    py_assert!(
        py,
        bytes_pybytes_conversion,
        "bytes_pybytes_conversion(b'Hello World') == b'Hello World'"
    );
}
