// Copyright (c) 2017-present PyO3 Project and Contributors

//! Context manager api
//! Trait and support implementation for context manager api
//!

use err::PyResult;
use python::{Python, PythonObject};
use class::methods::PyMethodDef;


/// Context manager interface
#[allow(unused_variables)]
pub trait PyContextProtocol: PythonObject {

    fn __enter__(&self, py: Python)
                  -> Self::Result where Self: PyContextEnterProtocol { unimplemented!() }

    fn __exit__(&self, py: Python,
                exc_type: Option<Self::ExcType>,
                exc_value: Option<Self::ExcValue>,
                traceback: Option<Self::Traceback>)
                -> Self::Result where Self: PyContextExitProtocol { unimplemented!() }
}

pub trait PyContextEnterProtocol: PyContextProtocol {
    type Success: ::ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyContextExitProtocol: PyContextProtocol {
    type ExcType: for<'a> ::FromPyObject<'a>;
    type ExcValue: for<'a> ::FromPyObject<'a>;
    type Traceback: for<'a> ::FromPyObject<'a>;
    type Success: ::ToPyObject;
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

impl<T> PyContextProtocolImpl for T where T: PyContextProtocol {
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

impl<T> PyContextEnterProtocolImpl for T
    where T: PyContextProtocol
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

impl<T> PyContextExitProtocolImpl for T
    where T: PyContextProtocol
{
    #[inline]
    default fn __exit__() -> Option<PyMethodDef> {
        None
    }
}
