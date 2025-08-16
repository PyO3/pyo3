use crate::{err::PyErrArguments, exceptions, PyErr, Python};
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
            } else {
                #[cfg(io_error_more)]
                #[allow(clippy::incompatible_msrv)] // gated by `io_error_more`
                if err.is_instance_of::<exceptions::PyIsADirectoryError>(py) {
                    io::ErrorKind::IsADirectory
                } else if err.is_instance_of::<exceptions::PyNotADirectoryError>(py) {
                    io::ErrorKind::NotADirectory
                } else {
                    io::ErrorKind::Other
                }
                #[cfg(not(io_error_more))]
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
            #[cfg(io_error_more)]
            io::ErrorKind::IsADirectory => exceptions::PyIsADirectoryError::new_err(err),
            #[cfg(io_error_more)]
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

impl_to_pyerr!(std::array::TryFromSliceError, exceptions::PyValueError);
impl_to_pyerr!(std::num::ParseIntError, exceptions::PyValueError);
impl_to_pyerr!(std::num::ParseFloatError, exceptions::PyValueError);
impl_to_pyerr!(std::num::TryFromIntError, exceptions::PyValueError);
impl_to_pyerr!(std::str::ParseBoolError, exceptions::PyValueError);
impl_to_pyerr!(std::ffi::IntoStringError, exceptions::PyUnicodeDecodeError);
impl_to_pyerr!(std::ffi::NulError, exceptions::PyValueError);
impl_to_pyerr!(std::str::Utf8Error, exceptions::PyUnicodeDecodeError);
impl_to_pyerr!(std::string::FromUtf8Error, exceptions::PyUnicodeDecodeError);
impl_to_pyerr!(
    std::string::FromUtf16Error,
    exceptions::PyUnicodeDecodeError
);
impl_to_pyerr!(
    std::char::DecodeUtf16Error,
    exceptions::PyUnicodeDecodeError
);
impl_to_pyerr!(std::net::AddrParseError, exceptions::PyValueError);

#[cfg(test)]
mod tests {
    use crate::{PyErr, Python};
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
        #[cfg(io_error_more)]
        check_err(io::ErrorKind::IsADirectory, "IsADirectoryError");
        #[cfg(io_error_more)]
        check_err(io::ErrorKind::NotADirectory, "NotADirectoryError");
    }
}
