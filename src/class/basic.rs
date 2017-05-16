// Copyright (c) 2017-present PyO3 Project and Contributors

//! Basic Python Object customization
//!
//! more information on python async support
//! https://docs.python.org/3/reference/datamodel.html#basic-customization

use ffi;
use err::{PyErr, PyResult};
use python::{self, Python, PythonObject};
use conversion::ToPyObject;
use objects::{PyObject, PyType, PyModule};
use py_class::slots::UnitCallbackConverter;
use function::{handle_callback, PyObjectCallbackConverter};
use class::NO_METHODS;

// __new__
// __init__
// __call__
// classmethod
// staticmethod


/// Basic customization
pub trait PyObjectProtocol {

    // fn __getattr__()
    // fn __setattr__()
    // fn __delattr__()
    // fn __getattribute__
    // fn __setattribute__
    // __instancecheck__
    // __subclasscheck__
    // __iter__
    // __next__

    fn __str__(&self, py: Python) -> PyResult<PyString>;

    fn __repr__(&self, py: Python) -> PyResult<PyString>;

    fn __hash__(&self, py: Python) -> PyResult<PyObject>;

    fn __bool__(&self, py: Python) -> PyResult<bool>;

    fn __richcmp__(&self, other: PyObject, op: pyo3::CompareOp) -> PyResult<bool>;

}
