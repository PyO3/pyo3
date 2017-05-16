// Copyright (c) 2017-present PyO3 Project and Contributors

#[macro_use] mod macros;

pub mod async;
pub mod basic;
pub mod buffer;
pub mod context;
pub mod descr;
pub mod mapping;
pub mod methods;
pub mod number;
pub mod gc;
pub mod sequence;
pub mod typeob;

pub use self::basic::PyObjectProtocol;
pub use self::async::PyAsyncProtocol;
pub use self::buffer::PyBufferProtocol;
pub use self::context::PyContextProtocol;
pub use self::descr::PyDescrProtocol;
pub use self::number::PyNumberProtocol;
pub use self::mapping::PyMappingProtocol;
pub use self::sequence::PySequenceProtocol;

pub use self::gc::{PyVisit, PyGCProtocol, PyTraverseError};
pub use self::methods::{PyMethodDef, PyMethodDefType, PyMethodType,
                        PyGetterDef, PySetterDef};

pub static NO_METHODS: &'static [&'static str] = &[];
pub static NO_PY_METHODS: &'static [PyMethodDefType] = &[];
