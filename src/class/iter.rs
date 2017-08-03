// Copyright (c) 2017-present PyO3 Project and Contributors
//! Python Iterator Interface.
//! Trait and support implementation for implementing iterators

use std::ptr;

use ffi;
use err::PyResult;
use python::{Python, IntoPyPointer};
use typeob::PyTypeInfo;
use conversion::IntoPyObject;
use callback::{CallbackConverter, PyObjectCallbackConverter};


/// Python Iterator Interface.
///
/// more information
/// `https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_iter`
#[allow(unused_variables)]
pub trait PyIterProtocol<'p> : PyTypeInfo {
    fn __iter__(&'p mut self)
                -> Self::Result where Self: PyIterIterProtocol<'p> { unimplemented!() }

    fn __next__(&'p mut self)
                -> Self::Result where Self: PyIterNextProtocol<'p> { unimplemented!() }

}

pub trait PyIterIterProtocol<'p>: PyIterProtocol<'p> {
    type Success: ::IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyIterNextProtocol<'p>: PyIterProtocol<'p> {
    type Success: ::IntoPyObject;
    type Result: Into<PyResult<Option<Self::Success>>>;
}


#[doc(hidden)]
pub trait PyIterProtocolImpl {
    fn tp_as_iter(typeob: &mut ffi::PyTypeObject);
}

impl<T> PyIterProtocolImpl for T {
    #[inline]
    default fn tp_as_iter(_: &mut ffi::PyTypeObject) {}
}

impl<'p, T> PyIterProtocolImpl for T where T: PyIterProtocol<'p> {
    #[inline]
    fn tp_as_iter(typeob: &mut ffi::PyTypeObject) {
        typeob.tp_iter = Self::tp_iter();
        typeob.tp_iternext = Self::tp_iternext();
    }
}

trait PyIterIterProtocolImpl {
    fn tp_iter() -> Option<ffi::getiterfunc>;
}

impl<'p, T> PyIterIterProtocolImpl for T where T: PyIterProtocol<'p>
{
    #[inline]
    default fn tp_iter() -> Option<ffi::getiterfunc> {
        None
    }
}

impl<T> PyIterIterProtocolImpl for T where T: for<'p> PyIterIterProtocol<'p>
{
    #[inline]
    fn tp_iter() -> Option<ffi::getiterfunc> {
        py_unary_func!(PyIterIterProtocol, T::__iter__, T::Success, PyObjectCallbackConverter)
    }
}

trait PyIterNextProtocolImpl {
    fn tp_iternext() -> Option<ffi::iternextfunc>;
}

impl<'p, T> PyIterNextProtocolImpl for T
    where T: PyIterProtocol<'p>
{
    #[inline]
    default fn tp_iternext() -> Option<ffi::iternextfunc> {
        None
    }
}

impl<T> PyIterNextProtocolImpl for T where T: for<'p> PyIterNextProtocol<'p>
{
    #[inline]
    fn tp_iternext() -> Option<ffi::iternextfunc> {
        py_unary_func!(PyIterNextProtocol, T::__next__,
                       Option<T::Success>, IterNextConverter)
    }
}


struct IterNextConverter;

impl <T> CallbackConverter<Option<T>> for IterNextConverter
    where T: IntoPyObject
{
    type R = *mut ffi::PyObject;

    fn convert(val: Option<T>, py: Python) -> *mut ffi::PyObject {
        match val {
            Some(val) => val.into_object(py).into_ptr(),
            None => unsafe {
                ffi::PyErr_SetNone(ffi::PyExc_StopIteration);
                ptr::null_mut()
            }
        }
    }

    #[inline]
    fn error_value() -> *mut ffi::PyObject {
        ptr::null_mut()
    }
}
