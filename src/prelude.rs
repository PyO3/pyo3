// Copyright (c) 2017-present PyO3 Project and Contributors

//! A collection of items you most likely want to have in scope when working with pyo3
//!
//! The purpose of this module is to alleviate imports of many common pyo3 traits
//! by adding a glob import to the top of pyo3 heavy modules:
//!
//! ```
//! # #![allow(unused_imports)]
//! use pyo3::prelude::*;
//! ```

pub use class::*;
pub use conversion::{
    FromPyObject, IntoPyObject, IntoPyTuple, PyTryFrom, PyTryInto, ToBorrowedObject, ToPyObject,
};
pub use err::{PyDowncastError, PyErr, PyErrArguments, PyErrValue, PyResult};
pub use instance::{AsPyRef, Py, PyNativeType, PyObjectWithToken, PyToken};
pub use noargs::NoArgs;
pub use object::PyObject;
pub use objectprotocol::ObjectProtocol;
pub use objects::*;
pub use python::{IntoPyPointer, Python, ToPyPointer};
pub use pythonrun::GILGuard;
pub use typeob::PyRawObject;

pub use pyo3cls::{pyclass, pyfunction, pymethods, pyproto};

#[cfg(Py_3)]
pub use pyo3cls::mod3init as pymodinit;

#[cfg(not(Py_3))]
pub use pyo3cls::mod2init as pymodinit;
