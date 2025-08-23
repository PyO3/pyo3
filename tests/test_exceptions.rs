#![cfg(feature = "macros")]

use pyo3::prelude::*;
use pyo3::{exceptions, py_run};
use std::error::Error;
use std::fmt;
#[cfg(not(target_os = "windows"))]
use std::fs::File;

mod test_utils;

#[pyfunction]
#[cfg(not(target_os = "windows"))]
fn fail_to_open_file() -> PyResult<()> {
    File::open("not_there.txt")?;
    Ok(())
}

#[test]
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
fn test_exception_nosegfault() {
    use std::net::TcpListener;
    fn io_err() -> PyResult<()> {
        TcpListener::bind("no:address")?;
        Ok(())
    }
    fn parse_int() -> PyResult<()> {
        "@_@".parse::<i64>()?;
        Ok(())
    }
    assert!(io_err().is_err());
    assert!(parse_int().is_err());
}

#[test]
#[cfg(all(Py_3_8, not(Py_GIL_DISABLED)))]
fn test_write_unraisable() {
    use pyo3::{exceptions::PyRuntimeError, ffi, types::PyNotImplemented};
    use std::ptr;
    use test_utils::UnraisableCapture;

    Python::attach(|py| {
        let capture = UnraisableCapture::install(py);

        assert!(capture.borrow(py).capture.is_none());

        let err = PyRuntimeError::new_err("foo");
        err.write_unraisable(py, None);

        let (err, object) = capture.borrow_mut(py).capture.take().unwrap();
        assert_eq!(err.to_string(), "RuntimeError: foo");
        assert!(object.is_none(py));

        let err = PyRuntimeError::new_err("bar");
        err.write_unraisable(py, Some(&PyNotImplemented::get(py)));

        let (err, object) = capture.borrow_mut(py).capture.take().unwrap();
        assert_eq!(err.to_string(), "RuntimeError: bar");
        assert!(unsafe { ptr::eq(object.as_ptr(), ffi::Py_NotImplemented()) });

        capture.borrow_mut(py).uninstall(py);
    });
}
