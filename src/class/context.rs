// Copyright (c) 2017-present PyO3 Project and Contributors

//! Context manager api
//! Trait and support implementation for context manager api
//!

use crate::class::methods::PyMethodDef;
use crate::err::PyResult;
use crate::type_object::PyTypeInfo;

/// Context manager interface
#[allow(unused_variables)]
pub trait PyContextProtocol<'p>: PyTypeInfo {
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
    type Success: crate::IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyContextExitProtocol<'p>: PyContextProtocol<'p> {
    type ExcType: crate::FromPyObject<'p>;
    type ExcValue: crate::FromPyObject<'p>;
    type Traceback: crate::FromPyObject<'p>;
    type Success: crate::IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

#[doc(hidden)]
pub trait PyContextProtocolImpl {
    fn methods() -> Vec<PyMethodDef> {
        Vec::new()
    }
}

impl<T> PyContextProtocolImpl for T {}

impl<'p, T> PyContextProtocolImpl for T
where
    T: PyContextProtocol<'p>,
{
    #[inline]
    fn methods() -> Vec<PyMethodDef> {
        let mut methods = Vec::new();

        if let Some(def) = <Self as PyContextEnterProtocolImpl>::__enter__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyContextExitProtocolImpl>::__exit__() {
            methods.push(def)
        }

        methods
    }
}

#[doc(hidden)]
pub trait PyContextEnterProtocolImpl {
    fn __enter__() -> Option<PyMethodDef> {
        None
    }
}

impl<'p, T> PyContextEnterProtocolImpl for T where T: PyContextProtocol<'p> {}

#[doc(hidden)]
pub trait PyContextExitProtocolImpl {
    fn __exit__() -> Option<PyMethodDef> {
        None
    }
}

impl<'p, T> PyContextExitProtocolImpl for T where T: PyContextProtocol<'p> {}
