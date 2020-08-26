//! Testing https://github.com/PyO3/pyo3/issues/1106. A result type that
//! implements `From<MyError> for PyErr` should be automatically converted
//! when using `#[pyfunction]`.
//!
//! This only makes sure that valid `Result` types do work. For an invalid
//! enum type, see `ui/invalid_result_conversion.py`.
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use std::fmt;

mod common;

/// A basic error type for the tests.
#[derive(Debug)]
struct MyError {
    pub descr: String,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "My error message: {}", self.descr)
    }
}

/// Important for the automatic conversion to `PyErr`.
impl From<MyError> for PyErr {
    fn from(err: MyError) -> pyo3::PyErr {
        pyo3::exceptions::PyOSError::py_err(err.to_string())
    }
}

#[pyfunction]
fn should_work() -> Result<(), MyError> {
    Err(MyError {
        descr: "something went wrong".to_string(),
    })
}

#[test]
fn test_result_conversion() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    wrap_pyfunction!(should_work)(py);
}
