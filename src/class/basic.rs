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


/// Basic customization
pub trait PyObjectProtocol {

    // fn __new__(&self, py: Python) -> PyResult<PyObject>;

    fn __str__(&self, py: Python) -> PyResult<PyString>;

    fn __repr__(&self, py: Python) -> PyResult<PyString>;

    fn __hash__(&self, py: Python) -> PyResult<PyObject>;

    fn __bool__(&self, py: Python) -> PyResult<bool>;

    fn __richcmp__(&self, other: PyObject, op: pyo3::CompareOp) -> PyResult<bool>;

    fn __call__(&self) -> PyResult<PyObject>;

}
