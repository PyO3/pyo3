// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Async/Await Interface
//! Trait and support implementation for implementing awaitable objects
//!
//! more information on python async support
//! https://docs.python.org/3/c-api/typeobj.html#async-object-structures

use ffi;
use err::PyResult;
use python::{Python, PythonObject};
use callback::PyObjectCallbackConverter;


/// Awaitable interface
#[allow(unused_variables)]
pub trait PyAsyncProtocol: PythonObject {

    fn __await__(&self, py: Python)
                 -> Self::Result where Self: PyAsyncAwaitProtocol { unimplemented!() }

    fn __aiter__(&self, py: Python)
                 -> Self::Result where Self: PyAsyncAiterProtocol { unimplemented!() }

    fn __anext__(&self, py: Python)
                 -> Self::Result where Self: PyAsyncAnextProtocol { unimplemented!() }

    fn __aenter__(&self, py: Python)
                  -> Self::Result where Self: PyAsyncAenterProtocol { unimplemented!() }

    fn __aexit__(&self, py: Python)
                 -> Self::Result where Self: PyAsyncAexitProtocol { unimplemented!() }

}


pub trait PyAsyncAwaitProtocol: PyAsyncProtocol {
    type Success: ::ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAiterProtocol: PyAsyncProtocol {
    type Success: ::ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAnextProtocol: PyAsyncProtocol {
    type Success: ::ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAenterProtocol: PyAsyncProtocol {
    type Success: ::ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAexitProtocol: PyAsyncProtocol {
    type Success: ::ToPyObject;
    type Result: Into<PyResult<Self::Success>>;
}


#[doc(hidden)]
pub trait PyAsyncProtocolImpl {
    fn tp_as_async() -> Option<ffi::PyAsyncMethods>;
}

impl<T> PyAsyncProtocolImpl for T {
    #[inline]
    default fn tp_as_async() -> Option<ffi::PyAsyncMethods> {
        None
    }
}

impl<T> PyAsyncProtocolImpl for T where T: PyAsyncProtocol {
    #[inline]
    fn tp_as_async() -> Option<ffi::PyAsyncMethods> {
        Some(ffi::PyAsyncMethods {
            am_await: Self::am_await(),
            am_aiter: Self::am_aiter(),
            am_anext: Self::am_anext(),
        })
    }
}


trait PyAsyncAwaitProtocolImpl {
    fn am_await() -> Option<ffi::unaryfunc>;
}

impl<T> PyAsyncAwaitProtocolImpl for T
    where T: PyAsyncProtocol
{
    #[inline]
    default fn am_await() -> Option<ffi::unaryfunc> {
        None
    }
}

impl<T> PyAsyncAwaitProtocolImpl for T
    where T: PyAsyncAwaitProtocol
{
    #[inline]
    fn am_await() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyAsyncAwaitProtocol, T::__await__, PyObjectCallbackConverter)
    }
}

trait PyAsyncAiterProtocolImpl {
    fn am_aiter() -> Option<ffi::unaryfunc>;
}

impl<T> PyAsyncAiterProtocolImpl for T
    where T: PyAsyncProtocol
{
    #[inline]
    default fn am_aiter() -> Option<ffi::unaryfunc> {
        None
    }
}

impl<T> PyAsyncAiterProtocolImpl for T
    where T: PyAsyncAiterProtocol
{
    #[inline]
    fn am_aiter() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyAsyncAiterProtocol, T::__aiter__, PyObjectCallbackConverter)
    }
}

trait PyAsyncAnextProtocolImpl {
    fn am_anext() -> Option<ffi::unaryfunc>;
}

impl<T> PyAsyncAnextProtocolImpl for T
    where T: PyAsyncProtocol
{
    #[inline]
    default fn am_anext() -> Option<ffi::unaryfunc> {
        None
    }
}

impl<T> PyAsyncAnextProtocolImpl for T
    where T: PyAsyncAnextProtocol
{
    #[inline]
    fn am_anext() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyAsyncAnextProtocol, T::__anext__, PyObjectCallbackConverter)
    }
}

trait PyAsyncAenterProtocolImpl {
    fn am_aenter() -> Option<ffi::unaryfunc>;
}

impl<T> PyAsyncAenterProtocolImpl for T
    where T: PyAsyncProtocol
{
    #[inline]
    default fn am_aenter() -> Option<ffi::unaryfunc> {
        None
    }
}

impl<T> PyAsyncAenterProtocolImpl for T
    where T: PyAsyncAenterProtocol
{
    #[inline]
    fn am_aenter() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyAsyncAenterProtocol, T::__aenter__, PyObjectCallbackConverter)
    }
}

trait PyAsyncAexitProtocolImpl {
    fn am_aexit() -> Option<ffi::unaryfunc>;
}

impl<T> PyAsyncAexitProtocolImpl for T
    where T: PyAsyncProtocol
{
    #[inline]
    default fn am_aexit() -> Option<ffi::unaryfunc> {
        None
    }
}

impl<T> PyAsyncAexitProtocolImpl for T
    where T: PyAsyncAexitProtocol
{
    #[inline]
    fn am_aexit() -> Option<ffi::unaryfunc> {
        py_unary_func_!(PyAsyncAexitProtocol, T::__aexit__, PyObjectCallbackConverter)
    }
}
