//! PyO3's prelude.
//!
//! The purpose of this module is to alleviate imports of many commonly used items of the PyO3 crate
//! by adding a glob import to the top of pyo3 heavy modules:
//!
//! ```
//! # #![allow(unused_imports)]
//! use pyo3::prelude::*;
//! ```

pub use crate::conversion::{FromPyObject, IntoPyObject};
#[allow(deprecated)]
pub use crate::conversion::{IntoPy, ToPyObject};
pub use crate::err::{PyErr, PyResult};
pub use crate::instance::{Borrowed, Bound, Py, PyObject};
pub use crate::marker::Python;
pub use crate::pycell::{PyRef, PyRefMut};
pub use crate::pyclass_init::PyClassInitializer;
pub use crate::types::{PyAny, PyModule};

#[cfg(feature = "macros")]
pub use pyo3_macros::{
    pyclass, pyfunction, pymethods, pymodule, FromPyObject, IntoPyObject, IntoPyObjectRef,
};

#[cfg(feature = "macros")]
pub use crate::wrap_pyfunction;
#[cfg(feature = "macros")]
#[allow(deprecated)]
pub use crate::wrap_pyfunction_bound;

pub use crate::types::any::PyAnyMethods;
pub use crate::types::weakref::PyWeakrefMethods;
