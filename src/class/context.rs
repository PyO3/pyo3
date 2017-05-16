// Copyright (c) 2017-present PyO3 Project and Contributors

//! Context manager api
//! Trait and support implementation for context manager api
//!

use err::PyResult;
use python::Python;
use objects::PyObject;
use class::{NO_METHODS, NO_PY_METHODS};


/// Awaitable interface
pub trait PyContextProtocol {

    fn __enter__(&self, py: Python) -> PyResult<PyObject>;

    fn __exit__(&self, py: Python,
                exc_type: Option<PyObject>,
                exc_value: Option<PyObject>,
                traceback: Option<PyObject>) -> PyResult<PyObject>;
}


impl<P> PyContextProtocol for P {

    default fn __enter__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.None())
    }

    default fn __exit__(&self, py: Python,
                        _exc_type: Option<PyObject>,
                        _exc_value: Option<PyObject>,
                        _traceback: Option<PyObject>) -> PyResult<PyObject> {
        Ok(py.None())
    }
}


#[doc(hidden)]
pub trait PyContextProtocolImpl {
    fn methods() -> &'static [&'static str];

    fn py_methods() -> &'static [::methods::PyMethodDefType];
}

impl<T> PyContextProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }
    default fn py_methods() -> &'static [::methods::PyMethodDefType] {
        NO_PY_METHODS
    }
}
