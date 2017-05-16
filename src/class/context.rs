// Copyright (c) 2017-present PyO3 Project and Contributors

//! Context manager api
//! Trait and support implementation for context manager api
//!

use ffi;
use err::{PyErr, PyResult};
use python::{self, Python, PythonObject};
use conversion::ToPyObject;
use objects::{PyObject, PyType, PyModule};
use py_class::slots::UnitCallbackConverter;
use function::{handle_callback, PyObjectCallbackConverter};
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

    fn py_methods() -> &'static [::class::PyMethodDef];
}

impl<T> PyContextProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }

    default fn py_methods() -> &'static [::class::PyMethodDef] {
        NO_PY_METHODS
    }
}

/*
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
*/
