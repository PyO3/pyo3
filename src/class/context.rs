// Copyright (c) 2017-present PyO3 Project and Contributors

//! Context manager api
//! Trait and support implementation for context manager api
//!

use crate::{callback::IntoPyCallbackOutput, derive_utils::TryFromPyCell};
use crate::{PyClass, PyObject};

/// Context manager interface
#[allow(unused_variables)]
pub trait PyContextProtocol<'p>: PyClass {
    fn __enter__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyContextEnterProtocol<'p>,
    {
        unimplemented!()
    }

    fn __exit__(
        slf: Self::Receiver,
        exc_type: Option<Self::ExcType>,
        exc_value: Option<Self::ExcValue>,
        traceback: Option<Self::Traceback>,
    ) -> Self::Result
    where
        Self: PyContextExitProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyContextEnterProtocol<'p>: PyContextProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyContextExitProtocol<'p>: PyContextProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type ExcType: crate::FromPyObject<'p>;
    type ExcValue: crate::FromPyObject<'p>;
    type Traceback: crate::FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}
