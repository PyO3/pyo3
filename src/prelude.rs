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

pub use conversion::{FromPyObject, IntoPyObject, PyTryFrom, PyTryInto, ToPyObject};
pub use err::{PyErr, PyResult};
pub use instance::{AsPyRef, Py, PyToken};
pub use noargs::NoArgs;
pub use object::PyObject;
pub use objectprotocol::ObjectProtocol;
pub use python::Python;
pub use pythonrun::GILGuard;
// This is only part of the prelude because we need it for the pymodinit function
pub use types::PyModule;
// This is required for the constructor
pub use PyRawObject;

pub use pyo3cls::{pyclass, pyfunction, pymethods, pyproto};

#[cfg(Py_3)]
pub use pyo3cls::mod3init as pymodinit;

#[cfg(not(Py_3))]
pub use pyo3cls::mod2init as pymodinit;
