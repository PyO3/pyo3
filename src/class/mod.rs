// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python object protocols

#[macro_use]
mod macros;

pub mod basic;
pub mod buffer;
pub mod context;
pub mod descr;
pub mod gc;
pub mod iter;
pub mod mapping;
pub mod methods;
pub mod number;
pub mod pyasync;
pub mod sequence;

pub use self::basic::PyObjectProtocol;
pub use self::buffer::PyBufferProtocol;
pub use self::context::PyContextProtocol;
pub use self::descr::PyDescrProtocol;
pub use self::gc::{PyGCProtocol, PyTraverseError, PyVisit};
pub use self::iter::PyIterProtocol;
pub use self::mapping::PyMappingProtocol;
pub use self::methods::{PyGetterDef, PyMethodDef, PyMethodDefType, PyMethodType, PySetterDef};
pub use self::number::PyNumberProtocol;
pub use self::pyasync::PyAsyncProtocol;
pub use self::sequence::PySequenceProtocol;
