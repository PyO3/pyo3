// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Mapping Interface
//! Trait and support implementation for implementing mapping support

use std::os::raw::c_int;

use ffi;
use err::{PyErr, PyResult};
use python::{self, Python, PythonObject};
use conversion::ToPyObject;
use objects::{exc, PyObject, PyType, PyModule};
use py_class::slots::{LenResultConverter, UnitCallbackConverter};
use function::{handle_callback, PyObjectCallbackConverter};
use class::NO_METHODS;


/// Mapping interface
pub trait PyMappingProtocol {
    fn __len__(&self, py: Python) -> PyResult<usize>;

    fn __getitem__(&self, py: Python, key: &PyObject) -> PyResult<PyObject>;

    fn __setitem__(&self, py: Python, key: &PyObject, value: Option<&PyObject>) -> PyResult<()>;
}

impl<T> PyMappingProtocol for T where T: PythonObject {
    default fn __len__(&self, _py: Python) -> PyResult<usize> {
        Ok(0)
    }

    default fn __getitem__(&self, py: Python, _: &PyObject) -> PyResult<PyObject> {
        Ok(py.None())
    }

    default fn __setitem__(&self, py: Python,
                           _: &PyObject, val: Option<&PyObject>) -> PyResult<()> {
        println!("=========");
        if let Some(_) = val {
            Err(PyErr::new::<exc::NotImplementedError, _>(
                py, format!("Subscript assignment not supported by {:?}", self.as_object())))
        } else {
            Err(PyErr::new::<exc::NotImplementedError, _>(
                py, format!("Subscript deletion not supported by {:?}", self.as_object())))
        }
    }
}

#[doc(hidden)]
pub trait PyMappingProtocolImpl {
    fn methods() -> &'static [&'static str];
}

impl<T> PyMappingProtocolImpl for T {
    default fn methods() -> &'static [&'static str] {
        NO_METHODS
    }
}

impl ffi::PyMappingMethods {

    /// Construct PyAsyncMethods struct for PyTypeObject.tp_as_mapping
    pub fn new<T>() -> Option<ffi::PyMappingMethods>
        where T: PyMappingProtocol + PyMappingProtocolImpl + PythonObject
    {
        let methods = T::methods();
        if methods.is_empty() {
            return None
        }

        let mut meth: ffi::PyMappingMethods = ffi::PyMappingMethods_INIT;

        for name in methods {
            match name {
                &"__len__" => {
                    meth.mp_length = py_unary_slot!(
                        PyMappingProtocol, T::__len__,
                        ffi::Py_ssize_t, LenResultConverter);
                },
                &"__getitem__" => {
                    meth.mp_subscript = py_binary_slot!(
                        PyMappingProtocol, T::__getitem__,
                        *mut ffi::PyObject, *mut ffi::PyObject, PyObjectCallbackConverter);
                },
                &"__setitem__" => {
                    meth.mp_ass_subscript = py_ternary_slot!(
                        PyMappingProtocol, T::__setitem__,
                        *mut ffi::PyObject, *mut ffi::PyObject, c_int,
                        UnitCallbackConverter);
                },
                _ => unreachable!(),
            }
        }

        // default method
        if ! methods.contains(&"__setitem__") {
            meth.mp_ass_subscript = py_ternary_slot!(
                PyMappingProtocol, T::__setitem__,
                *mut ffi::PyObject, *mut ffi::PyObject, c_int,
                UnitCallbackConverter);
        }

        Some(meth)
    }
}
