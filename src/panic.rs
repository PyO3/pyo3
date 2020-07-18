use crate::exceptions::PyBaseException;

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
