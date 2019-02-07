// Copyright (c) 2017-present PyO3 Project and Contributors

use crate::conversion::{IntoPy, IntoPyObject, ToPyObject};
use crate::instance::Py;
use crate::object::PyObject;
use crate::python::Python;
use crate::types::PyTuple;

/// An empty struct that represents the empty argument list.
/// Corresponds to the empty tuple `()` in Python.
///
/// # Example
/// ```
/// # use pyo3::prelude::*;
///
/// let gil = Python::acquire_gil();
/// let py = gil.python();
/// let os = py.import("os").unwrap();
/// let pid = os.call("get_pid", NoArgs, None);
/// ```
#[derive(Copy, Clone, Debug)]
pub struct NoArgs;

/// Converts `NoArgs` to an empty Python tuple.
impl IntoPy<Py<PyTuple>> for NoArgs {
    fn into_py(self, py: Python) -> Py<PyTuple> {
        PyTuple::empty(py)
    }
}

/// Converts `()` to an empty Python tuple.
impl IntoPy<Py<PyTuple>> for () {
    fn into_py(self, py: Python) -> Py<PyTuple> {
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
