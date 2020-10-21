use crate::exceptions::PyBaseException;
use crate::PyErr;
use std::any::Any;

pyo3_exception!(
    "
    The exception raised when Rust code called from Python panics.

    Like SystemExit, this exception is derived from BaseException so that
    it will typically propagate all the way through the stack and cause the
    Python interpreter to exit.
    ",
    PanicException,
    PyBaseException
);

impl PanicException {
    // Try to format the error in the same way panic does
    pub(crate) fn from_panic(e: Box<dyn Any + Send + 'static>) -> PyErr {
        if let Some(string) = e.downcast_ref::<String>() {
            Self::new_err((string.clone(),))
        } else if let Some(s) = e.downcast_ref::<&str>() {
            Self::new_err((s.to_string(),))
        } else {
            Self::new_err(("panic from Rust code",))
        }
    }
}
