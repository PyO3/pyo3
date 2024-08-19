//! Testing https://github.com/PyO3/pyo3/issues/1106. A result type that
//! *doesn't* implement `From<MyError> for PyErr` won't be automatically
//! converted when using `#[pyfunction]`.
use pyo3::prelude::*;

use std::fmt;

/// A basic error type for the tests. It's missing `From<MyError> for PyErr`,
/// though, so it shouldn't work.
#[derive(Debug)]
struct MyError {
    pub descr: &'static str,
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "My error message: {}", self.descr)
    }
}

#[pyfunction]
fn should_not_work() -> Result<(), MyError> {
    Err(MyError {
        descr: "something went wrong",
    })
}

fn main() {
    Python::with_gil(|py| {
        wrap_pyfunction!(should_not_work)(py);
    });
}
