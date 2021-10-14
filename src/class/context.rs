// Copyright (c) 2017-present PyO3 Project and Contributors

//! Context manager api
//! Trait and support implementation for context manager api

use crate::callback::IntoPyCallbackOutput;
use crate::{PyClass, PyObject};

/// Context manager interface
#[allow(unused_variables)]
pub trait PyContextProtocol<'p>: PyClass {
    #[deprecated(
        since = "0.14.0",
        note = "prefer implementing `__enter__` in `#[pymethods]` instead of in a protocol"
    )]
    fn __enter__(&'p mut self) -> Self::Result
    where
        Self: PyContextEnterProtocol<'p>,
    {
        unimplemented!()
    }

    #[deprecated(
        since = "0.14.0",
        note = "prefer implementing `__exit__` in `#[pymethods]` instead of in a protocol"
    )]
    fn __exit__(
        &'p mut self,
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
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyContextExitProtocol<'p>: PyContextProtocol<'p> {
    type ExcType: crate::FromPyObject<'p>;
    type ExcValue: crate::FromPyObject<'p>;
    type Traceback: crate::FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}
