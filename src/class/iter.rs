// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Iterator Interface
//! Trait and support implementation for implementing iterators
//!
//! more information
//! https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_iter

use ffi;
use err::PyResult;
use python::{Python, PythonObject};
use callback::PyObjectCallbackConverter;


/// Iterator protocol
#[allow(unused_variables)]
pub trait PyIterProtocol : PythonObject {
    fn __iter__(&self, py: Python)
                 -> Self::Result where Self: PyIterIterProtocol { unimplemented!() }

    fn __next__(&self, py: Python)
                -> Self::Result where Self: PyIterNextProtocol { unimplemented!() }

}

pub trait PyIterIterProtocol: PyIterProtocol {
    type Success: ::ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyIterNextProtocol: PyIterProtocol {
    type Success: ::ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}


#[doc(hidden)]
pub trait PyIterProtocolImpl {
    fn tp_as_iter(typeob: &mut ffi::PyTypeObject);
}

impl<T> PyIterProtocolImpl for T {
    #[inline]
    default fn tp_as_iter(_: &mut ffi::PyTypeObject) {}
}

impl<T> PyIterProtocolImpl for T where T: PyIterProtocol {
    #[inline]
    fn tp_as_iter(typeob: &mut ffi::PyTypeObject) {
        typeob.tp_iter = Self::tp_iter();
        typeob.tp_iternext = Self::tp_iternext();
    }
}

trait PyIterIterProtocolImpl {
    fn tp_iter() -> Option<ffi::getiterfunc>;
}

impl<T> PyIterIterProtocolImpl for T
    where T: PyIterProtocol
{
    #[inline]
    default fn tp_iter() -> Option<ffi::getiterfunc> {
        None
    }
}

impl<T> PyIterIterProtocolImpl for T
    where T: PyIterIterProtocol
{
    #[inline]
    fn tp_iter() -> Option<ffi::getiterfunc> {
        py_unary_func_!(PyIterIterProtocol, T::__iter__, PyObjectCallbackConverter)
    }
}

trait PyIterNextProtocolImpl {
    fn tp_iternext() -> Option<ffi::iternextfunc>;
}

impl<T> PyIterNextProtocolImpl for T
    where T: PyIterProtocol
{
    #[inline]
    default fn tp_iternext() -> Option<ffi::iternextfunc> {
        None
    }
}

impl<T> PyIterNextProtocolImpl for T
    where T: PyIterNextProtocol
{
    #[inline]
    fn tp_iternext() -> Option<ffi::iternextfunc> {
        py_unary_func_!(PyIterNextProtocol, T::__next__, PyObjectCallbackConverter)
    }
}
