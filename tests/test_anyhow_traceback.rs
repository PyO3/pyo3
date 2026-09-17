#![cfg(all(
    feature = "anyhow",
    feature = "macros",
    not(Py_LIMITED_API),
    not(PyPy),
    not(GraalPy)
))]

//! `anyhow` only captures a backtrace when `RUST_BACKTRACE` is set,
//! and `std` caches that decision on first use.
//! So this is the only test in its binary:
//! it can set the variable before anything reads it, without racing other tests.

use anyhow::Context;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyTracebackMethods;

fn failing() -> anyhow::Result<()> {
    Err(PyValueError::new_err("Value Error").into())
}

#[pyfunction]
fn rust_anyhow() -> anyhow::Result<()> {
    failing().context("Context")
}

#[test]
fn test_traceback() {
    // SAFETY: sole test in this binary, so no other thread can be reading the environment
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    Python::attach(|py| {
        let err = wrap_pyfunction!(rust_anyhow, py)
            .unwrap()
            .call0()
            .unwrap_err();
        let formatted = err
            .traceback(py)
            .expect("expected traceback")
            .format()
            .expect("expected formatting to work");

        // see `test_backtrace.rs` for how the whole traceback looks
        assert!(
            formatted.contains("test_anyhow_traceback::failing"),
            "{formatted}"
        );
        assert!(
            formatted.contains("test_anyhow_traceback::rust_anyhow"),
            "{formatted}"
        );
        // trimmed at the FFI boundary, so the test harness below it is gone
        assert!(!formatted.contains("test::run_test"), "{formatted}");
    })
}
