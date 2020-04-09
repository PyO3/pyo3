// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Mapping Interface
//! Trait and support implementation for implementing mapping support

use crate::class::methods::PyMethodDef;
use crate::err::{PyErr, PyResult};
use crate::{exceptions, ffi, FromPyObject, IntoPy, PyClass, PyObject};

/// Mapping interface
#[allow(unused_variables)]
pub trait PyMappingProtocol<'p>: PyClass {
    fn __len__(&'p self) -> Self::Result
    where
        Self: PyMappingLenProtocol<'p>,
    {
        unimplemented!()
    }

    fn __getitem__(&'p self, key: Self::Key) -> Self::Result
    where
        Self: PyMappingGetItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __setitem__(&'p mut self, key: Self::Key, value: Self::Value) -> Self::Result
    where
        Self: PyMappingSetItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __delitem__(&'p mut self, key: Self::Key) -> Self::Result
    where
        Self: PyMappingDelItemProtocol<'p>,
    {
        unimplemented!()
    }

    fn __reversed__(&'p self) -> Self::Result
    where
        Self: PyMappingReversedProtocol<'p>,
    {
        unimplemented!()
    }
}

// The following are a bunch of marker traits used to detect
// the existance of a slotted method.

pub trait PyMappingLenProtocol<'p>: PyMappingProtocol<'p> {
    type Result: Into<PyResult<usize>>;
}

pub trait PyMappingGetItemProtocol<'p>: PyMappingProtocol<'p> {
    type Key: FromPyObject<'p>;
    type Success: IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyMappingSetItemProtocol<'p>: PyMappingProtocol<'p> {
    type Key: FromPyObject<'p>;
    type Value: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}

pub trait PyMappingDelItemProtocol<'p>: PyMappingProtocol<'p> {
    type Key: FromPyObject<'p>;
    type Result: Into<PyResult<()>>;
}

pub trait PyMappingReversedProtocol<'p>: PyMappingProtocol<'p> {
    type Success: IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

#[doc(hidden)]
pub trait PyMappingProtocolImpl {
    fn tp_as_mapping() -> Option<ffi::PyMappingMethods>;
    fn methods() -> Vec<PyMethodDef>;
}

impl<T> PyMappingProtocolImpl for T {
    default fn tp_as_mapping() -> Option<ffi::PyMappingMethods> {
        None
    }
    default fn methods() -> Vec<PyMethodDef> {
        Vec::new()
    }
}

impl<'p, T> PyMappingProtocolImpl for T
where
    T: PyMappingProtocol<'p>,
{
    #[inline]
    fn tp_as_mapping() -> Option<ffi::PyMappingMethods> {
        let f = if let Some(df) = Self::mp_del_subscript() {
            Some(df)
        } else {
            Self::mp_ass_subscript()
        };

        Some(ffi::PyMappingMethods {
            mp_length: Self::mp_length(),
            mp_subscript: Self::mp_subscript(),
            mp_ass_subscript: f,
        })
    }

    #[inline]
    fn methods() -> Vec<PyMethodDef> {
        let mut methods = Vec::new();

        if let Some(def) = <Self as PyMappingReversedProtocolImpl>::__reversed__() {
            methods.push(def)
        }

        methods
    }
}

trait PyMappingLenProtocolImpl {
    fn mp_length() -> Option<ffi::lenfunc>;
}

impl<'p, T> PyMappingLenProtocolImpl for T
where
    T: PyMappingProtocol<'p>,
{
    default fn mp_length() -> Option<ffi::lenfunc> {
        None
    }
}

impl<T> PyMappingLenProtocolImpl for T
where
    T: for<'p> PyMappingLenProtocol<'p>,
{
    #[inline]
    fn mp_length() -> Option<ffi::lenfunc> {
        py_len_func!(PyMappingLenProtocol, T::__len__)
    }
}

trait PyMappingGetItemProtocolImpl {
    fn mp_subscript() -> Option<ffi::binaryfunc>;
}

impl<'p, T> PyMappingGetItemProtocolImpl for T
where
    T: PyMappingProtocol<'p>,
{
    default fn mp_subscript() -> Option<ffi::binaryfunc> {
        None
    }
}

impl<T> PyMappingGetItemProtocolImpl for T
where
    T: for<'p> PyMappingGetItemProtocol<'p>,
{
    #[inline]
    fn mp_subscript() -> Option<ffi::binaryfunc> {
        py_binary_func!(PyMappingGetItemProtocol, T::__getitem__)
    }
}

trait PyMappingSetItemProtocolImpl {
    fn mp_ass_subscript() -> Option<ffi::objobjargproc>;
}

impl<'p, T> PyMappingSetItemProtocolImpl for T
where
    T: PyMappingProtocol<'p>,
{
    default fn mp_ass_subscript() -> Option<ffi::objobjargproc> {
        None
    }
}

impl<T> PyMappingSetItemProtocolImpl for T
where
    T: for<'p> PyMappingSetItemProtocol<'p>,
{
    #[inline]
    fn mp_ass_subscript() -> Option<ffi::objobjargproc> {
        py_func_set!(PyMappingSetItemProtocol, T, __setitem__)
    }
}

/// Returns `None` if PyMappingDelItemProtocol isn't implemented, otherwise dispatches to
/// `DelSetItemDispatch`
trait DeplItemDipatch {
    fn mp_del_subscript() -> Option<ffi::objobjargproc>;
}

impl<'p, T> DeplItemDipatch for T
where
    T: PyMappingProtocol<'p>,
{
    default fn mp_del_subscript() -> Option<ffi::objobjargproc> {
        None
    }
}

/// Returns `py_func_set_del` if PyMappingSetItemProtocol is implemented, otherwise `py_func_del`
trait DelSetItemDispatch: Sized + for<'p> PyMappingDelItemProtocol<'p> {
    fn det_set_dispatch() -> Option<ffi::objobjargproc>;
}

impl<T> DelSetItemDispatch for T
where
    T: Sized + for<'p> PyMappingDelItemProtocol<'p>,
{
    default fn det_set_dispatch() -> Option<ffi::objobjargproc> {
        py_func_del!(PyMappingDelItemProtocol, Self, __delitem__)
    }
}

impl<T> DelSetItemDispatch for T
where
    T: for<'p> PyMappingSetItemProtocol<'p> + for<'p> PyMappingDelItemProtocol<'p>,
{
    fn det_set_dispatch() -> Option<ffi::objobjargproc> {
        py_func_set_del!(
            PyMappingSetItemProtocol,
            PyMappingDelItemProtocol,
            T,
            __setitem__,
            __delitem__
        )
    }
}

impl<T> DeplItemDipatch for T
where
    T: Sized + for<'p> PyMappingDelItemProtocol<'p>,
{
    fn mp_del_subscript() -> Option<ffi::objobjargproc> {
        <T as DelSetItemDispatch>::det_set_dispatch()
    }
}

#[doc(hidden)]
pub trait PyMappingReversedProtocolImpl {
    fn __reversed__() -> Option<PyMethodDef>;
}

impl<'p, T> PyMappingReversedProtocolImpl for T
where
    T: PyMappingProtocol<'p>,
{
    default fn __reversed__() -> Option<PyMethodDef> {
        None
    }
}
