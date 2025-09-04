#![cfg(feature = "macros")]

use pyo3::prelude::*;

mod test_utils;

#[pyfunction]
fn take_str(_s: &str) {}

#[test]
fn test_unicode_encode_error() {
    Python::attach(|py| {
        let take_str = wrap_pyfunction!(take_str)(py).unwrap();
        py_expect_exception!(
            py,
            take_str,
            "take_str('\\ud800')",
            PyUnicodeEncodeError,
            "'utf-8' codec can't encode character '\\ud800' in position 0: surrogates not allowed"
        );
    });
}
