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

pub use crate::conversion::{FromPyObject, IntoPyObject, PyTryFrom, PyTryInto, ToPyObject};
pub use crate::err::{PyErr, PyResult};
pub use crate::instance::{AsPyRef, Py};
pub use crate::noargs::NoArgs;
pub use crate::object::PyObject;
pub use crate::objectprotocol::ObjectProtocol;
pub use crate::python::Python;
pub use crate::pythonrun::GILGuard;
// This is only part of the prelude because we need it for the pymodule function
pub use crate::types::PyModule;
// This is required for the constructor
pub use crate::PyRawObject;

pub use pyo3cls::{pyclass, pyfunction, pymethods, pyproto};

#[cfg(Py_3)]
pub use pyo3cls::mod3init as pymodule;

#[cfg(not(Py_3))]
pub use pyo3cls::mod2init as pymodule;
