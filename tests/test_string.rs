#![cfg(feature = "macros")]

use pyo3::prelude::*;

#[path = "../src/tests/common.rs"]
mod common;

#[pyfunction]
fn take_str(_s: &str) {}

#[test]
fn test_unicode_encode_error() {
    Python::with_gil(|py| {
        let take_str = wrap_pyfunction_bound!(take_str)(py).unwrap();
        py_expect_exception!(
            py,
            take_str,
            "take_str('\\ud800')",
            PyUnicodeEncodeError,
            "'utf-8' codec can't encode character '\\ud800' in position 0: surrogates not allowed"
        );
    });
}
