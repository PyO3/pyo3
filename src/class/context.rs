// Copyright (c) 2017-present PyO3 Project and Contributors

//! Context manager api
//! Trait and support implementation for context manager api
//!

use err::PyResult;
use typeob::PyTypeInfo;
use class::methods::PyMethodDef;


/// Context manager interface
#[allow(unused_variables)]
pub trait PyContextProtocol<'p>: PyTypeInfo {

    fn __enter__(&'p mut self)
                 -> Self::Result where Self: PyContextEnterProtocol<'p> {unimplemented!()}

    fn __exit__(&'p mut self,
                exc_type: Option<Self::ExcType>,
                exc_value: Option<Self::ExcValue>,
                traceback: Option<Self::Traceback>)
                -> Self::Result where Self: PyContextExitProtocol<'p> { unimplemented!() }
}

pub trait PyContextEnterProtocol<'p>: PyContextProtocol<'p> {
    type Success: ::IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyContextExitProtocol<'p>: PyContextProtocol<'p> {
    type ExcType: ::FromPyObject<'p>;
    type ExcValue: ::FromPyObject<'p>;
    type Traceback: ::FromPyObject<'p>;
    type Success: ::IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

#[doc(hidden)]
pub trait PyContextProtocolImpl {
    fn methods() -> Vec<PyMethodDef>;
}

impl<T> PyContextProtocolImpl for T {
    #[inline]
    default fn methods() -> Vec<PyMethodDef> {
        Vec::new()
    }
}

impl<'p, T> PyContextProtocolImpl for T where T: PyContextProtocol<'p> {
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
    fn __enter__() -> Option<PyMethodDef>;
}

impl<'p, T> PyContextEnterProtocolImpl for T where T: PyContextProtocol<'p>
{
    #[inline]
    default fn __enter__() -> Option<PyMethodDef> {
        None
    }
}

#[doc(hidden)]
pub trait PyContextExitProtocolImpl {
    fn __exit__() -> Option<PyMethodDef>;
}

impl<'p, T> PyContextExitProtocolImpl for T where T: PyContextProtocol<'p>
{
    #[inline]
    default fn __exit__() -> Option<PyMethodDef> {
        None
    }
}
