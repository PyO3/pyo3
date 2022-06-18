#![allow(deprecated)]
// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python object protocols

#[macro_use]
mod macros;

pub mod basic;
#[cfg(any(not(Py_LIMITED_API), Py_3_11))]
pub mod buffer;
pub mod descr;
pub mod gc;
pub mod iter;
pub mod mapping;
#[doc(hidden)]
pub use crate::impl_::pymethods as methods;
pub mod number;
pub mod pyasync;
pub mod sequence;

pub use self::basic::PyObjectProtocol;
#[cfg(any(not(Py_LIMITED_API), Py_3_11))]
pub use self::buffer::PyBufferProtocol;
pub use self::descr::PyDescrProtocol;
pub use self::gc::{PyGCProtocol, PyTraverseError, PyVisit};
pub use self::iter::PyIterProtocol;
pub use self::mapping::PyMappingProtocol;
#[doc(hidden)]
pub use self::methods::{PyClassAttributeDef, PyGetterDef, PyMethodDef, PyMethodType, PySetterDef};
pub use self::number::PyNumberProtocol;
pub use self::pyasync::PyAsyncProtocol;
pub use self::sequence::PySequenceProtocol;
