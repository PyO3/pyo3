use crate::{err::PyErrArguments, exceptions, types, PyErr, Python};
use crate::{IntoPyObject, Py, PyAny};
use std::io;

/// Convert `PyErr` to `io::Error`
impl From<PyErr> for io::Error {
    fn from(err: PyErr) -> Self {
        let kind = Python::attach(|py| {
            if err.is_instance_of::<exceptions::PyBrokenPipeError>(py) {
                io::ErrorKind::BrokenPipe
            } else if err.is_instance_of::<exceptions::PyConnectionRefusedError>(py) {
                io::ErrorKind::ConnectionRefused
            } else if err.is_instance_of::<exceptions::PyConnectionAbortedError>(py) {
                io::ErrorKind::ConnectionAborted
            } else if err.is_instance_of::<exceptions::PyConnectionResetError>(py) {
                io::ErrorKind::ConnectionReset
            } else if err.is_instance_of::<exceptions::PyInterruptedError>(py) {
                io::ErrorKind::Interrupted
            } else if err.is_instance_of::<exceptions::PyFileNotFoundError>(py) {
                io::ErrorKind::NotFound
            } else if err.is_instance_of::<exceptions::PyPermissionError>(py) {
                io::ErrorKind::PermissionDenied
            } else if err.is_instance_of::<exceptions::PyFileExistsError>(py) {
                io::ErrorKind::AlreadyExists
            } else if err.is_instance_of::<exceptions::PyBlockingIOError>(py) {
                io::ErrorKind::WouldBlock
            } else if err.is_instance_of::<exceptions::PyTimeoutError>(py) {
                io::ErrorKind::TimedOut
            } else if err.is_instance_of::<exceptions::PyMemoryError>(py) {
                io::ErrorKind::OutOfMemory
            } else if err.is_instance_of::<exceptions::PyIsADirectoryError>(py) {
                io::ErrorKind::IsADirectory
            } else if err.is_instance_of::<exceptions::PyNotADirectoryError>(py) {
                io::ErrorKind::NotADirectory
            } else {
                io::ErrorKind::Other
            }
        });
        io::Error::new(kind, err)
    }
}

/// Create `PyErr` from `io::Error`
/// (`OSError` except if the `io::Error` is wrapping a Python exception,
/// in this case the exception is returned)
impl From<io::Error> for PyErr {
    fn from(err: io::Error) -> PyErr {
        // If the error wraps a Python error we return it
        if err.get_ref().is_some_and(|e| e.is::<PyErr>()) {
            return *err.into_inner().unwrap().downcast().unwrap();
        }
        match err.kind() {
            io::ErrorKind::BrokenPipe => exceptions::PyBrokenPipeError::new_err(err),
            io::ErrorKind::ConnectionRefused => exceptions::PyConnectionRefusedError::new_err(err),
            io::ErrorKind::ConnectionAborted => exceptions::PyConnectionAbortedError::new_err(err),
            io::ErrorKind::ConnectionReset => exceptions::PyConnectionResetError::new_err(err),
            io::ErrorKind::Interrupted => exceptions::PyInterruptedError::new_err(err),
            io::ErrorKind::NotFound => exceptions::PyFileNotFoundError::new_err(err),
            io::ErrorKind::PermissionDenied => exceptions::PyPermissionError::new_err(err),
            io::ErrorKind::AlreadyExists => exceptions::PyFileExistsError::new_err(err),
            io::ErrorKind::WouldBlock => exceptions::PyBlockingIOError::new_err(err),
            io::ErrorKind::TimedOut => exceptions::PyTimeoutError::new_err(err),
            io::ErrorKind::OutOfMemory => exceptions::PyMemoryError::new_err(err),
            io::ErrorKind::IsADirectory => exceptions::PyIsADirectoryError::new_err(err),
            io::ErrorKind::NotADirectory => exceptions::PyNotADirectoryError::new_err(err),
            _ => exceptions::PyOSError::new_err(err),
        }
    }
}

impl PyErrArguments for io::Error {
    fn arguments(self, py: Python<'_>) -> Py<PyAny> {
        //FIXME(icxolu) remove unwrap
        self.to_string()
            .into_pyobject(py)
            .unwrap()
            .into_any()
            .unbind()
    }
}

impl<W> From<io::IntoInnerError<W>> for PyErr {
    fn from(err: io::IntoInnerError<W>) -> PyErr {
        err.into_error().into()
    }
}

impl<W: Send + Sync> PyErrArguments for io::IntoInnerError<W> {
    fn arguments(self, py: Python<'_>) -> Py<PyAny> {
        self.into_error().arguments(py)
    }
}

impl From<std::convert::Infallible> for PyErr {
    fn from(_: std::convert::Infallible) -> PyErr {
        unreachable!()
    }
}

macro_rules! impl_to_pyerr {
    ($err: ty, $pyexc: ty) => {
        impl PyErrArguments for $err {
            fn arguments(self, py: Python<'_>) -> $crate::Py<$crate::PyAny> {
                // FIXME(icxolu) remove unwrap
                self.to_string()
                    .into_pyobject(py)
                    .unwrap()
                    .into_any()
                    .unbind()
            }
        }

        impl std::convert::From<$err> for PyErr {
            fn from(err: $err) -> PyErr {
                <$pyexc>::new_err(err)
            }
        }
    };
}

pub(crate) struct Utf8ErrorWithBytes {
    pub(crate) err: std::str::Utf8Error,
    pub(crate) bytes: Vec<u8>,
}

impl PyErrArguments for Utf8ErrorWithBytes {
    fn arguments(self, py: Python<'_>) -> Py<PyAny> {
        let Self { err, bytes } = self;
        let start = err.valid_up_to();
        let end = err.error_len().map_or(bytes.len(), |l| start + l);

        let encoding = types::PyString::new(py, "utf-8").into_any();
        let bytes = types::PyBytes::new(py, &bytes).into_any();
        let start = types::PyInt::new(py, start).into_any();
        let end = types::PyInt::new(py, end).into_any();
        let reason = types::PyString::new(py, "invalid utf-8").into_any();

        // FIXME(icxolu) remove unwrap
        types::PyTuple::new(py, &[encoding, bytes, start, end, reason])
            .unwrap()
            .into_any()
            .unbind()
    }
}

impl std::convert::From<Utf8ErrorWithBytes> for PyErr {
    fn from(err: Utf8ErrorWithBytes) -> PyErr {
        exceptions::PyUnicodeDecodeError::new_err(err)
    }
}

impl PyErrArguments for std::string::FromUtf8Error {
    fn arguments(self, py: Python<'_>) -> Py<PyAny> {
        Utf8ErrorWithBytes {
            err: self.utf8_error(),
            bytes: self.into_bytes(),
        }
        .arguments(py)
    }
}

impl std::convert::From<std::string::FromUtf8Error> for PyErr {
    fn from(err: std::string::FromUtf8Error) -> PyErr {
        exceptions::PyUnicodeDecodeError::new_err(err)
    }
}

impl PyErrArguments for std::ffi::IntoStringError {
    fn arguments(self, py: Python<'_>) -> Py<PyAny> {
        Utf8ErrorWithBytes {
            err: self.utf8_error(),
            bytes: self.into_cstring().into_bytes(),
        }
        .arguments(py)
    }
}

impl std::convert::From<std::ffi::IntoStringError> for PyErr {
    fn from(err: std::ffi::IntoStringError) -> PyErr {
        exceptions::PyUnicodeDecodeError::new_err(err)
    }
}

impl_to_pyerr!(std::array::TryFromSliceError, exceptions::PyValueError);
impl_to_pyerr!(std::num::ParseIntError, exceptions::PyValueError);
impl_to_pyerr!(std::num::ParseFloatError, exceptions::PyValueError);
impl_to_pyerr!(std::num::TryFromIntError, exceptions::PyValueError);
impl_to_pyerr!(std::str::ParseBoolError, exceptions::PyValueError);
impl_to_pyerr!(std::ffi::NulError, exceptions::PyValueError);
impl_to_pyerr!(std::net::AddrParseError, exceptions::PyValueError);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{types::PyAnyMethods, IntoPyObjectExt, PyErr, Python};
    use std::io;

    #[test]
    fn io_errors() {
        use crate::types::any::PyAnyMethods;

        let check_err = |kind, expected_ty| {
            Python::attach(|py| {
                let rust_err = io::Error::new(kind, "some error msg");

                let py_err: PyErr = rust_err.into();
                let py_err_msg = format!("{expected_ty}: some error msg");
                assert_eq!(py_err.to_string(), py_err_msg);
                let py_error_clone = py_err.clone_ref(py);

                let rust_err_from_py_err: io::Error = py_err.into();
                assert_eq!(rust_err_from_py_err.to_string(), py_err_msg);
                assert_eq!(rust_err_from_py_err.kind(), kind);

                let py_err_recovered_from_rust_err: PyErr = rust_err_from_py_err.into();
                assert!(py_err_recovered_from_rust_err
                    .value(py)
                    .is(py_error_clone.value(py))); // It should be the same exception
            })
        };

        check_err(io::ErrorKind::BrokenPipe, "BrokenPipeError");
        check_err(io::ErrorKind::ConnectionRefused, "ConnectionRefusedError");
        check_err(io::ErrorKind::ConnectionAborted, "ConnectionAbortedError");
        check_err(io::ErrorKind::ConnectionReset, "ConnectionResetError");
        check_err(io::ErrorKind::Interrupted, "InterruptedError");
        check_err(io::ErrorKind::NotFound, "FileNotFoundError");
        check_err(io::ErrorKind::PermissionDenied, "PermissionError");
        check_err(io::ErrorKind::AlreadyExists, "FileExistsError");
        check_err(io::ErrorKind::WouldBlock, "BlockingIOError");
        check_err(io::ErrorKind::TimedOut, "TimeoutError");
        check_err(io::ErrorKind::IsADirectory, "IsADirectoryError");
        check_err(io::ErrorKind::NotADirectory, "NotADirectoryError");
    }

    #[test]
    #[allow(invalid_from_utf8)]
    fn utf8_errors() {
        let bytes = b"abc\xffdef".to_vec();

        let check_err = |py_err: PyErr| {
            Python::attach(|py| {
                let py_err = py_err.into_bound_py_any(py).unwrap();

                assert!(py_err.is_instance_of::<exceptions::PyUnicodeDecodeError>());
                assert_eq!(
                    py_err
                        .getattr("encoding")
                        .unwrap()
                        .extract::<String>()
                        .unwrap(),
                    "utf-8"
                );
                assert_eq!(
                    py_err
                        .getattr("object")
                        .unwrap()
                        .extract::<Vec<u8>>()
                        .unwrap(),
                    &*bytes
                );
                assert_eq!(
                    py_err.getattr("start").unwrap().extract::<usize>().unwrap(),
                    3
                );
                assert_eq!(
                    py_err.getattr("end").unwrap().extract::<usize>().unwrap(),
                    4
                );
                assert_eq!(
                    py_err
                        .getattr("reason")
                        .unwrap()
                        .extract::<String>()
                        .unwrap(),
                    "invalid utf-8"
                );
            });
        };

        let utf8_err_with_bytes = Utf8ErrorWithBytes {
            err: std::str::from_utf8(&bytes).expect_err("\\xff is invalid utf-8"),
            bytes: bytes.clone(),
        }
        .into();
        check_err(utf8_err_with_bytes);

        let from_utf8_err = String::from_utf8(bytes.clone())
            .expect_err("\\xff is invalid utf-8")
            .into();
        check_err(from_utf8_err);

        let from_utf8_err = std::ffi::CString::new(bytes.clone())
            .unwrap()
            .into_string()
            .expect_err("\\xff is invalid utf-8")
            .into();
        check_err(from_utf8_err);
    }
}
