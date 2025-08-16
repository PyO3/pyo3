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
pub use crate::err::{PyErr, PyResult};
#[allow(deprecated)]
pub use crate::instance::PyObject;
pub use crate::instance::{Borrowed, Bound, Py};
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

pub use crate::types::any::PyAnyMethods;
pub use crate::types::boolobject::PyBoolMethods;
pub use crate::types::bytearray::PyByteArrayMethods;
pub use crate::types::bytes::PyBytesMethods;
pub use crate::types::capsule::PyCapsuleMethods;
pub use crate::types::complex::PyComplexMethods;
pub use crate::types::dict::PyDictMethods;
pub use crate::types::float::PyFloatMethods;
pub use crate::types::frozenset::PyFrozenSetMethods;
pub use crate::types::list::PyListMethods;
pub use crate::types::mapping::PyMappingMethods;
pub use crate::types::mappingproxy::PyMappingProxyMethods;
pub use crate::types::module::PyModuleMethods;
pub use crate::types::sequence::PySequenceMethods;
pub use crate::types::set::PySetMethods;
pub use crate::types::slice::PySliceMethods;
pub use crate::types::string::PyStringMethods;
pub use crate::types::traceback::PyTracebackMethods;
pub use crate::types::tuple::PyTupleMethods;
pub use crate::types::typeobject::PyTypeMethods;
pub use crate::types::weakref::PyWeakrefMethods;
