// Copyright (c) 2017-present PyO3 Project and Contributors

//! Context manager api
//! Trait and support implementation for context manager api
//!

use crate::err::PyResult;
use crate::{Py, PyClass, PyObject};

/// Context manager interface
#[allow(unused_variables)]
pub trait PyContextProtocol<'p>: PyClass {
    fn __enter__(&'p mut self) -> Self::Result
    where
        Self: PyContextEnterProtocol<'p>,
    {
        unimplemented!()
    }

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
    type Success: crate::IntoPy<Py<PyObject>>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyContextExitProtocol<'p>: PyContextProtocol<'p> {
    type ExcType: crate::FromPyObject<'p>;
    type ExcValue: crate::FromPyObject<'p>;
    type Traceback: crate::FromPyObject<'p>;
    type Success: crate::IntoPy<Py<PyObject>>;
    type Result: Into<PyResult<Self::Success>>;
}
