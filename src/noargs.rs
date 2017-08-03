// Copyright (c) 2017-present PyO3 Project and Contributors

use std;

use ffi;
use instance::Py;
use python::{Python, IntoPyDictPointer};
use conversion::{ToPyObject, IntoPyObject, IntoPyTuple};
use object::PyObject;
use objects::PyTuple;

/// An empty struct that represents the empty argument list.
/// Corresponds to the empty tuple `()` in Python.
///
/// # Example
/// ```
/// let gil = pyo3::Python::acquire_gil();
/// let py = gil.python();
/// let os = py.import("os").unwrap();
/// let pid = os.call("get_pid", pyo3::NoArgs, pyo3::NoArgs);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct NoArgs;

/// Converts `NoArgs` to an empty Python tuple.
impl IntoPyTuple for NoArgs {

    fn into_tuple(self, py: Python) -> Py<PyTuple> {
        PyTuple::empty(py)
    }
}

/// Converts `()` to an empty Python tuple.
impl IntoPyTuple for () {

    fn into_tuple(self, py: Python) -> Py<PyTuple> {
        PyTuple::empty(py)
    }
}

/// Converts `NoArgs` to an empty Python tuple.
impl ToPyObject for NoArgs {

    fn to_object(&self, py: Python) -> PyObject {
        PyTuple::empty(py).into()
    }
}

/// Converts `NoArgs` to an empty Python tuple.
impl IntoPyObject for NoArgs {

    fn into_object(self, py: Python) -> PyObject {
        PyTuple::empty(py).into()
    }
}

/// Converts `NoArgs` to an null pointer.
impl IntoPyDictPointer for NoArgs {

    fn into_dict_ptr(self, _: Python) -> *mut ffi::PyObject {
        std::ptr::null_mut()
    }
}

/// Converts `()` to an null pointer.
impl IntoPyDictPointer for () {

    fn into_dict_ptr(self, _: Python) -> *mut ffi::PyObject {
        std::ptr::null_mut()
    }
}
