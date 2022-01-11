// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Async/Await Interface.
//!
//! Check [the Python C API information](
//! https://docs.python.org/3/c-api/typeobj.html#async-object-structures)
//!
//! [PEP-0492](https://www.python.org/dev/peps/pep-0492/)

use crate::callback::IntoPyCallbackOutput;
use crate::derive_utils::TryFromPyCell;
use crate::{PyClass, PyObject};

/// Python Async/Await support interface.
///
/// Each method in this trait corresponds to Python async/await implementation.
#[allow(unused_variables)]
pub trait PyAsyncProtocol<'p>: PyClass {
    fn __await__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyAsyncAwaitProtocol<'p>,
    {
        unimplemented!()
    }

    fn __aiter__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyAsyncAiterProtocol<'p>,
    {
        unimplemented!()
    }

    fn __anext__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyAsyncAnextProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyAsyncAwaitProtocol<'p>: PyAsyncProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyAsyncAiterProtocol<'p>: PyAsyncProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyAsyncAnextProtocol<'p>: PyAsyncProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyIterANextOutput>;
}

py_unarys_func!(await_, PyAsyncAwaitProtocol, Self::__await__);
py_unarys_func!(aiter, PyAsyncAiterProtocol, Self::__aiter__);
py_unarys_func!(anext, PyAsyncAnextProtocol, Self::__anext__);

pub use crate::pyclass::{IterANextOutput, PyIterANextOutput};
