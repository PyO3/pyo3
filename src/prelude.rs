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

pub use crate::err::{PyErr, PyResult};
pub use crate::gil::GILGuard;
pub use crate::instance::{AsPyRef, Py, PyRef, PyRefMut};
pub use crate::object::PyObject;
pub use crate::objectprotocol::ObjectProtocol;
pub use crate::python::Python;
pub use crate::{
    FromPy, FromPyObject, IntoPy, IntoPyObject, IntoPyPointer, PyTryFrom, PyTryInto, ToPyObject,
};
// This is only part of the prelude because we need it for the pymodule function
pub use crate::types::PyModule;
// This is required for the constructor
pub use crate::PyRawObject;
pub use pyo3cls::pymodule;
pub use pyo3cls::{pyclass, pyfunction, pymethods, pyproto};
