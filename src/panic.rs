//! Helper to convert Rust panics to Python exceptions.
#[cfg(not(feature = "panicexception-inherits-exception"))]
use crate::exceptions::PyBaseException;
#[cfg(feature = "panicexception-inherits-exception")]
use crate::exceptions::PyException;
use crate::PyErr;
use std::any::Any;

/// Helper macro to factorize PanicException's documentation
#[macro_export]
macro_rules! panic_exception_doc {
    () => {
        "
The exception raised when Rust code called from Python panics.

By default, like SystemExit, this exception is derived from BaseException so that
it will typically propagate all the way through the stack and cause the
Python interpreter to exit.

However this behavior can be changed by enabling the `panicexception-inherits-exception`
feature: this exception will then derived from Exception.
"
    };
}

#[cfg(not(feature = "panicexception-inherits-exception"))]
pyo3_exception!(panic_exception_doc!(), PanicException, PyBaseException);
#[cfg(feature = "panicexception-inherits-exception")]
pyo3_exception!(panic_exception_doc!(), PanicException, PyException);

impl PanicException {
    /// Creates a new PanicException from a panic payload.
    ///
    /// Attempts to format the error in the same way panic does.
    #[cold]
    pub(crate) fn from_panic_payload(payload: Box<dyn Any + Send + 'static>) -> PyErr {
        if let Some(string) = payload.downcast_ref::<String>() {
            Self::new_err((string.clone(),))
        } else if let Some(s) = payload.downcast_ref::<&str>() {
            Self::new_err((s.to_string(),))
        } else {
            Self::new_err(("panic from Rust code",))
        }
    }
}
