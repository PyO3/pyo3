#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::types::PyBytes;

mod test_utils;

#[pyfunction]
fn bytes_pybytes_conversion(bytes: &[u8]) -> &[u8] {
    bytes
}

#[test]
fn test_pybytes_bytes_conversion() {
    Python::attach(|py| {
        let f = wrap_pyfunction!(bytes_pybytes_conversion)(py).unwrap();
        py_assert!(py, f, "f(b'Hello World') == b'Hello World'");
    });
}

#[pyfunction]
fn bytes_vec_conversion(py: Python<'_>, bytes: Vec<u8>) -> Bound<'_, PyBytes> {
    PyBytes::new(py, bytes.as_slice())
}

#[test]
fn test_pybytes_vec_conversion() {
    Python::attach(|py| {
        let f = wrap_pyfunction!(bytes_vec_conversion)(py).unwrap();
        py_assert!(py, f, "f(b'Hello World') == b'Hello World'");
    });
}

#[test]
fn test_bytearray_vec_conversion() {
    Python::attach(|py| {
        let f = wrap_pyfunction!(bytes_vec_conversion)(py).unwrap();
        py_assert!(py, f, "f(bytearray(b'Hello World')) == b'Hello World'");
    });
}

#[test]
fn test_py_as_bytes() {
    let pyobj: pyo3::Py<pyo3::types::PyBytes> =
        Python::attach(|py| pyo3::types::PyBytes::new(py, b"abc").unbind());

    let data = Python::attach(|py| pyobj.as_bytes(py));

    assert_eq!(data, b"abc");

    Python::attach(move |_py| drop(pyobj));
}
