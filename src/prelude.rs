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
pub use objects::*;
pub use objectprotocol::ObjectProtocol;
pub use object::PyObject;
pub use noargs::NoArgs;
pub use python::{Python, ToPyPointer, IntoPyPointer};
pub use err::{PyErr, PyErrValue, PyResult, PyDowncastError, PyErrArguments};
pub use pythonrun::GILGuard;
pub use typeob::PyRawObject;
pub use instance::{PyToken, PyObjectWithToken, AsPyRef, Py, PyNativeType};
pub use conversion::{FromPyObject, PyTryFrom, PyTryInto,
                     ToPyObject, ToBorrowedObject, IntoPyObject, IntoPyTuple};

pub use pyo3cls::{pyproto, pyclass, pymethods, pyfunction};

#[cfg(Py_3)]
pub use pyo3cls::mod3init as pymodinit;

#[cfg(not(Py_3))]
pub use pyo3cls::mod2init as pymodinit;

// Until the extern macro story is fully fleshed out, this will make wrap_function! work
//#[doc(hidden)]
//pub use mashup::*;
