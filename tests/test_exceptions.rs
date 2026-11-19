#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::{exceptions, py_run};
use std::error::Error;
use std::fmt;
#[cfg(wip_feature_std)]
#[cfg(not(target_os = "windows"))]
use std::fs::File;

mod test_utils;

#[cfg(wip_feature_std)]
#[pyfunction]
#[cfg(not(target_os = "windows"))]
fn fail_to_open_file() -> PyResult<()> {
    File::open("not_there.txt")?;
    Ok(())
}

#[test]
#[cfg(wip_feature_std)]
#[cfg_attr(target_arch = "wasm32", ignore)] // Not sure why this fails.
#[cfg(not(target_os = "windows"))]
fn test_filenotfounderror() {
    Python::attach(|py| {
        let fail_to_open_file = wrap_pyfunction!(fail_to_open_file)(py).unwrap();

        py_run!(
            py,
            fail_to_open_file,
            r#"
        try:
            fail_to_open_file()
        except FileNotFoundError as e:
            assert str(e) == "No such file or directory (os error 2)"
        "#
        );
    });
}

#[derive(Debug)]
struct CustomError;

impl Error for CustomError {}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Oh no!")
    }
}

impl std::convert::From<CustomError> for PyErr {
    fn from(err: CustomError) -> PyErr {
        exceptions::PyOSError::new_err(err.to_string())
    }
}

fn fail_with_custom_error() -> Result<(), CustomError> {
    Err(CustomError)
}

#[pyfunction]
fn call_fail_with_custom_error() -> PyResult<()> {
    fail_with_custom_error()?;
    Ok(())
}

#[test]
fn test_custom_error() {
    Python::attach(|py| {
        let call_fail_with_custom_error =
            wrap_pyfunction!(call_fail_with_custom_error)(py).unwrap();

        py_run!(
            py,
            call_fail_with_custom_error,
            r#"
        try:
            call_fail_with_custom_error()
        except OSError as e:
            assert str(e) == "Oh no!"
        "#
        );
    });
}

#[test]
#[cfg(not(Py_LIMITED_API))]
fn test_pyfunction_traceback() {
    use pyo3::types::PyDict;

    #[pyfunction]
    fn fail_from_rust() -> PyResult<()> {
        Err(exceptions::PyValueError::new_err("from Rust"))
    }

    Python::attach(|py| {
        let locals = PyDict::new(py);
        locals
            .set_item(
                "fail_from_rust",
                wrap_pyfunction!(fail_from_rust, py).unwrap(),
            )
            .unwrap();

        py.run(
            c"def call_from_python():\n    fail_from_rust()",
            Some(&locals),
            Some(&locals),
        )
        .unwrap();

        let err = locals
            .get_item("call_from_python")
            .unwrap()
            .unwrap()
            .call0()
            .unwrap_err();

        locals.set_item("tb", err.traceback(py)).unwrap();

        let frames: Vec<String> = py
            .eval(
                c"[frame.line for frame in __import__('traceback').extract_tb(tb)]",
                Some(&locals),
                Some(&locals),
            )
            .unwrap()
            .extract()
            .unwrap();

        assert_eq!(
            frames,
            vec![
                // Rust -> Python
                ".call0()",
                // Python -> Rust
                "",
                // Rust entrypoint
                "fn fail_from_rust() -> PyResult<()> {",
                // Exception raised in Rust
                "Err(exceptions::PyValueError::new_err(\"from Rust\"))"
            ]
        );
    });
}

#[test]
fn test_exception_nosegfault() {
    #[cfg(wip_feature_std)]
    use std::net::TcpListener;
    #[cfg(wip_feature_std)]
    fn io_err() -> PyResult<()> {
        TcpListener::bind("no:address")?;
        Ok(())
    }
    fn parse_int() -> PyResult<()> {
        "@_@".parse::<i64>()?;
        Ok(())
    }
    #[cfg(wip_feature_std)]
    assert!(io_err().is_err());
    assert!(parse_int().is_err());
}

#[test]
fn test_write_unraisable() {
    use pyo3::{exceptions::PyRuntimeError, types::PyNotImplemented};
    use test_utils::UnraisableCapture;

    Python::attach(|py| {
        UnraisableCapture::enter(py, |capture| {
            let err = PyRuntimeError::new_err("foo");
            err.write_unraisable(py, None);

            let (err, object) = capture.take_capture().unwrap();

            assert_eq!(err.to_string(), "RuntimeError: foo");
            assert!(object.is_none());

            let err = PyRuntimeError::new_err("bar");
            err.write_unraisable(py, Some(&PyNotImplemented::get(py)));

            let (err, object) = capture.take_capture().unwrap();

            assert_eq!(err.to_string(), "RuntimeError: bar");
            assert!(object.is(PyNotImplemented::get(py)));
        });
    });
}
