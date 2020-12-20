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
pub use crate::instance::{Py, PyObject};
pub use crate::pycell::{PyCell, PyRef, PyRefMut};
pub use crate::pyclass_init::PyClassInitializer;
pub use crate::python::Python;
pub use crate::{FromPyObject, IntoPy, IntoPyPointer, PyTryFrom, PyTryInto, ToPyObject};
// PyModule is only part of the prelude because we need it for the pymodule function
pub use crate::types::{PyAny, PyModule};
#[cfg(feature = "macros")]
pub use pyo3_macros::{pyclass, pyfunction, pymethods, pymodule, pyproto, FromPyObject};
