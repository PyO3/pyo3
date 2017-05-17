// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Async/Await Interface
//! Trait and support implementation for implementing awaitable objects
//!
//! more information on python async support
//! https://docs.python.org/3/c-api/typeobj.html#async-object-structures

use ffi;
use err::PyResult;
use python::{Python, PythonObject};
use objects::PyObject;
use callback::PyObjectCallbackConverter;
use class::NO_METHODS;


/// Awaitable interface
pub trait PyAsyncProtocol {

    fn __await__(&self, py: Python) -> PyResult<PyObject>;

    fn __aiter__(&self, py: Python) -> PyResult<PyObject>;

    fn __anext__(&self, py: Python) -> PyResult<PyObject>;

    fn __aenter__(&self, py: Python) -> PyResult<PyObject>;

    fn __aexit__(&self, py: Python) -> PyResult<PyObject>;

}


impl<P> PyAsyncProtocol for P {

    default fn __await__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.None())
    }

    default fn __aiter__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.None())
    }

    default fn __anext__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.None())
    }

    default fn __aenter__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.None())
    }

    default fn __aexit__(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.None())
    }
}


#[doc(hidden)]
pub trait PyAsyncProtocolImpl {
    fn methods() -> &'static [&'static str];
}

impl<T> PyAsyncProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }
}

impl ffi::PyAsyncMethods {

    /// Construct PyAsyncMethods struct for PyTypeObject.tp_as_async
    pub fn new<T>() -> Option<ffi::PyAsyncMethods>
        where T: PyAsyncProtocol + PyAsyncProtocolImpl + PythonObject
    {
        let methods = T::methods();
        if methods.is_empty() {
            return None
        }

        let mut meth: ffi::PyAsyncMethods = ffi::PyAsyncMethods_INIT;

        for name in methods {
            match name {
                &"__await__" => {
                    meth.am_await = py_unary_func!(
                        PyAsyncProtocol, T::__await__, PyObjectCallbackConverter);
                },
                &"__aiter__" => {
                    meth.am_aiter = py_unary_func!(
                        PyAsyncProtocol, T::__aiter__, PyObjectCallbackConverter);
                },
                &"__anext__" => {
                    meth.am_anext = py_unary_func!(
                        PyAsyncProtocol, T::__anext__, PyObjectCallbackConverter);
                },
                _ => unreachable!(),
            }
        }

        Some(meth)
    }
}
