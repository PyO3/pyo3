// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python object protocols

#[macro_use] mod macros;

pub mod async;
pub mod basic;
pub mod buffer;
pub mod context;
pub mod descr;
pub mod mapping;
pub mod methods;
pub mod number;
pub mod iter;
pub mod gc;
pub mod sequence;

pub use self::basic::PyObjectProtocol;
pub use self::async::PyAsyncProtocol;
pub use self::iter::PyIterProtocol;
pub use self::buffer::PyBufferProtocol;
pub use self::context::PyContextProtocol;
pub use self::descr::PyDescrProtocol;
pub use self::number::PyNumberProtocol;
pub use self::mapping::PyMappingProtocol;
pub use self::sequence::PySequenceProtocol;

pub use self::gc::{PyVisit, PyGCProtocol, PyTraverseError};
pub use self::methods::{PyMethodDef, PyMethodDefType, PyMethodType,
                        PyGetterDef, PySetterDef};

use ffi;

#[derive(Debug)]
pub enum CompareOp {
    Lt = ffi::Py_LT as isize,
    Le = ffi::Py_LE as isize,
    Eq = ffi::Py_EQ as isize,
    Ne = ffi::Py_NE as isize,
    Gt = ffi::Py_GT as isize,
    Ge = ffi::Py_GE as isize
}
