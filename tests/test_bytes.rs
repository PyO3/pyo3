#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::types::PyBytes;

mod common;

#[pyfunction]
fn bytes_pybytes_conversion(bytes: &[u8]) -> &[u8] {
    bytes
}

#[test]
fn test_pybytes_bytes_conversion() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let f = wrap_pyfunction!(bytes_pybytes_conversion)(py).unwrap();
    py_assert!(py, f, "f(b'Hello World') == b'Hello World'");
}

#[pyfunction]
fn bytes_vec_conversion(py: Python, bytes: Vec<u8>) -> &PyBytes {
    PyBytes::new(py, bytes.as_slice())
}

#[test]
fn test_pybytes_vec_conversion() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let f = wrap_pyfunction!(bytes_vec_conversion)(py).unwrap();
    py_assert!(py, f, "f(b'Hello World') == b'Hello World'");
}

#[test]
fn test_bytearray_vec_conversion() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let f = wrap_pyfunction!(bytes_vec_conversion)(py).unwrap();
    py_assert!(py, f, "f(bytearray(b'Hello World')) == b'Hello World'");
}

#[test]
fn test_py_as_bytes() {
    let pyobj: pyo3::Py<pyo3::types::PyBytes>;
    let data: &[u8];

    {
        let gil = Python::acquire_gil();
        let py = gil.python();
        pyobj = pyo3::types::PyBytes::new(py, b"abc").into_py(py);
        data = pyobj.as_bytes(py);
    }

    assert_eq!(data, b"abc");
}
