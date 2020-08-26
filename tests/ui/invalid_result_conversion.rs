//! Testing https://github.com/PyO3/pyo3/issues/1106. A result type that
//! *doesn't* implement `From<MyError> for PyErr` won't be automatically
//! converted when using `#[pyfunction]`.
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

use std::fmt;

mod common;

#[derive(Debug)]
enum MyError {
    Custom(String),
    Unexpected(String),
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MyError::*;
        match self {
            Custom(e) => write!(f, "My error message: {}", e),
            Unexpected(e) => write!(f, "Unexpected: {}", e),
        }
    }
}

#[pyfunction]
fn should_not_work() -> Result<(), MyError> {
}

#[test]
fn test_result_conversion() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let f = wrap_pyfunction!(should_not_work)(py);
}
