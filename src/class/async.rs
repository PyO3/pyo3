// Copyright (c) 2017-present PyO3 Project and Contributors

//! Represent Python Async Object Structures
//! Trait and support implementation for implementing awaitable objects
//!
//! more information on python async support
//! https://docs.python.org/3/c-api/typeobj.html#async-object-structures

use ffi;
use err::{PyErr, PyResult};
use python::{self, Python, PythonObject};
use conversion::ToPyObject;
use objects::{PyObject, PyType, PyModule};
use py_class::slots::UnitCallbackConverter;
use function::{handle_callback, PyObjectCallbackConverter};
use class::NO_METHODS;


/// Awaitable interface
pub trait PyAsyncProtocol {

    fn am_await(&self, py: Python) -> PyResult<PyObject>;

    fn am_aiter(&self, py: Python) -> PyResult<PyObject>;

    fn am_anext(&self, py: Python) -> PyResult<PyObject>;

}


impl<P> PyAsyncProtocol for P {

    default fn am_await(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.None())
    }

    default fn am_aiter(&self, py: Python) -> PyResult<PyObject> {
        Ok(py.None())
    }

    default fn am_anext(&self, py: Python) -> PyResult<PyObject> {
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
                &"am_await" => {
                    meth.am_await = py_unary_slot!(
                        PyAsyncProtocol, T::am_await,
                        *mut ffi::PyObject, PyObjectCallbackConverter);
                },
                &"am_aiter" => {
                    meth.am_aiter = py_unary_slot!(
                        PyAsyncProtocol, T::am_aiter,
                        *mut ffi::PyObject, PyObjectCallbackConverter);
                },
                &"am_anext" => {
                    meth.am_anext = py_unary_slot!(
                        PyAsyncProtocol, T::am_anext,
                        *mut ffi::PyObject, PyObjectCallbackConverter);
                },
                _ => unreachable!(),
            }
        }

        Some(meth)
    }
}
