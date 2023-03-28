//! Helper to convert Rust panics to Python exceptions.
use crate::conversion::{FromPyPointer, IntoPyPointer};
use crate::exceptions::PyBaseException;
use crate::ffi;
use crate::global_api::ensure_global_api;
use crate::{PyAny, Python};
use std::any::Any;
use std::slice;
use std::str;

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
    /// Creates a new PanicException from a panic payload.
    ///
    /// Attempts to format the error in the same way panic does.
    #[cold]
    pub(crate) fn from_panic_payload<'py>(
        py: Python<'py>,
        payload: Box<dyn Any + Send + 'static>,
    ) -> &'py PyAny {
        let msg = if let Some(string) = payload.downcast_ref::<String>() {
            string.clone()
        } else if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else {
            "panic from Rust code".to_owned()
        };

        let api = match ensure_global_api(py) {
            Ok(api) => api,
            // The global API is unavailable, hence we fall back to our own `PanicException`.
            Err(err) => return PanicException::new_err((msg,)).into_value(py).into_ref(py),
        };

        let err = (api.create_panic_exception)(msg.as_ptr(), msg.len());

        PyAny::from_owned_ptr(py, err)
    }
}

pub(crate) unsafe extern "C" fn create_panic_exception(
    msg_ptr: *const u8,
    msg_len: usize,
) -> *mut ffi::PyObject {
    let msg = str::from_utf8_unchecked(slice::from_raw_parts(msg_ptr, msg_len));

    let err = PanicException::new_err((msg,));

    err.into_value(Python::assume_gil_acquired()).into_ptr()
}
