use crate::{err::PyErrArguments, exceptions, IntoPy, PyErr, PyObject, Python};
use std::io;

/// Convert `PyErr` to `io::Error`
impl std::convert::From<PyErr> for io::Error {
    fn from(err: PyErr) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("Python exception: {}", err))
    }
}

/// Create `OSError` from `io::Error`
impl std::convert::From<io::Error> for PyErr {
    fn from(err: io::Error) -> PyErr {
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
            _ => exceptions::PyOSError::new_err(err),
        }
    }
}

impl PyErrArguments for io::Error {
    fn arguments(self, py: Python<'_>) -> PyObject {
        self.to_string().into_py(py)
    }
}

impl<W> From<io::IntoInnerError<W>> for PyErr {
    fn from(err: io::IntoInnerError<W>) -> PyErr {
        err.into_error().into()
    }
}

impl<W: Send + Sync> PyErrArguments for io::IntoInnerError<W> {
    fn arguments(self, py: Python<'_>) -> PyObject {
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
            fn arguments(self, py: Python<'_>) -> PyObject {
                self.to_string().into_py(py)
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
    use crate::PyErr;
    use std::io;

    #[test]
    fn io_errors() {
        let check_err = |kind, expected_ty| {
            let py_err: PyErr = io::Error::new(kind, "some error msg").into();
            let err_msg = format!("{}: some error msg", expected_ty);
            assert_eq!(py_err.to_string(), err_msg);

            let os_err: io::Error = py_err.into();
            assert_eq!(os_err.to_string(), format!("Python exception: {}", err_msg));
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
    }
}
