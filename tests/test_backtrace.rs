#![cfg(all(
    feature = "macros",
    feature = "btparse",
    not(Py_LIMITED_API),
    not(PyPy),
    not(GraalPy)
))]

use std::backtrace::Backtrace;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTracebackMethods};

/// Line of the `#[pyfunction]` attribute below, which the generated wrapper is attributed to.
const WRAPPER_LINE: u32 = line!() + 2;
/// Innermost Rust frame: attaches its own backtrace as a traceback.
#[pyfunction]
fn rust_inner(py: Python<'_>) -> PyResult<()> {
    let err = PyRuntimeError::new_err("boom");
    err.set_traceback(py, Some(Backtrace::force_capture().into_pyobject(py)?));
    Err(err)
}
/// Line of the `force_capture` call above.
const CAPTURE_LINE: u32 = WRAPPER_LINE + 3;

/// Outermost Rust frame, calls back into Python.
#[pyfunction]
fn rust_outer(callback: &Bound<'_, PyAny>) -> PyResult<()> {
    callback.call0()?;
    Ok(())
}

/// Python -> Rust -> Python -> Rust: the traceback covers the innermost Rust segment plus the
/// Python frames the interpreter adds on the way out, but not the Rust frames on the far side
/// of those Python ones.
#[test]
fn test_nested_traceback() {
    let formatted = Python::attach(|py| {
        let globals = PyDict::new(py);
        globals
            .set_item("rust_inner", wrap_pyfunction!(rust_inner, py).unwrap())
            .unwrap();
        globals
            .set_item("rust_outer", wrap_pyfunction!(rust_outer, py).unwrap())
            .unwrap();

        let err = py
            .run(
                c"
def py_mid():
    rust_inner()

rust_outer(py_mid)
",
                Some(&globals),
                None,
            )
            .unwrap_err();

        err.traceback(py)
            .expect("expected traceback")
            .format()
            .unwrap()
    });

    assert_eq!(
        formatted,
        format!(
            r#"Traceback (most recent call last):
  File "<string>", line 5, in <module>
  File "<string>", line 3, in py_mid
  File "./tests/test_backtrace.rs", line {WRAPPER_LINE}, in test_backtrace::__pyfunction_rust_inner
    #[pyfunction]
  File "./tests/test_backtrace.rs", line {CAPTURE_LINE}, in test_backtrace::rust_inner
    err.set_traceback(py, Some(Backtrace::force_capture().into_pyobject(py)?));
"#
        )
    )
}
