// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Mapping Interface
//! Trait and support implementation for implementing mapping support

use crate::callback::{LenResultConverter, PyObjectCallbackConverter};
use crate::class::methods::PyMethodDef;
use crate::err::{PyErr, PyResult};
use crate::exceptions;
use crate::ffi;
use crate::type_object::PyTypeInfo;
use crate::Python;
use crate::{FromPyObject, IntoPyObject};

/// Mapping interface
#[allow(unused_variables)]
pub trait PyMappingProtocol<'p>: PyTypeInfo {
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

    fn __iter__(&'p self, py: Python<'p>) -> Self::Result
    where
        Self: PyMappingIterProtocol<'p>,
    {
        unimplemented!()
    }

    fn __contains__(&'p self, value: Self::Value) -> Self::Result
    where
        Self: PyMappingContainsProtocol<'p>,
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
    type Success: IntoPyObject;
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

pub trait PyMappingIterProtocol<'p>: PyMappingProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyMappingContainsProtocol<'p>: PyMappingProtocol<'p> {
    type Value: FromPyObject<'p>;
    type Result: Into<PyResult<bool>>;
}

pub trait PyMappingReversedProtocol<'p>: PyMappingProtocol<'p> {
    type Success: IntoPyObject;
    type Result: Into<PyResult<Self::Success>>;
}

#[doc(hidden)]
pub trait PyMappingProtocolImpl {
    fn tp_as_mapping() -> Option<ffi::PyMappingMethods> {
        None
    }
    fn methods() -> Vec<PyMethodDef> {
        Vec::new()
    }
}

impl<T> PyMappingProtocolImpl for T {}

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

        if let Some(def) = <Self as PyMappingIterProtocolImpl>::__iter__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyMappingContainsProtocolImpl>::__contains__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyMappingReversedProtocolImpl>::__reversed__() {
            methods.push(def)
        }

        methods
    }
}

trait PyMappingLenProtocolImpl {
    fn mp_length() -> Option<ffi::lenfunc> {
        None
    }
}

impl<'p, T> PyMappingLenProtocolImpl for T where T: PyMappingProtocol<'p> {}

impl<T> PyMappingLenProtocolImpl for T
where
    T: for<'p> PyMappingLenProtocol<'p>,
{
    #[inline]
    fn mp_length() -> Option<ffi::lenfunc> {
        py_len_func!(PyMappingLenProtocol, T::__len__, LenResultConverter)
    }
}

trait PyMappingGetItemProtocolImpl {
    fn mp_subscript() -> Option<ffi::binaryfunc> {
        None
    }
}

impl<'p, T> PyMappingGetItemProtocolImpl for T where T: PyMappingProtocol<'p> {}

impl<T> PyMappingGetItemProtocolImpl for T
where
    T: for<'p> PyMappingGetItemProtocol<'p>,
{
    #[inline]
    fn mp_subscript() -> Option<ffi::binaryfunc> {
        py_binary_func!(
            PyMappingGetItemProtocol,
            T::__getitem__,
            T::Success,
            PyObjectCallbackConverter
        )
    }
}

trait PyMappingSetItemProtocolImpl {
    fn mp_ass_subscript() -> Option<ffi::objobjargproc> {
        None
    }
}

impl<'p, T> PyMappingSetItemProtocolImpl for T where T: PyMappingProtocol<'p> {}

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
    fn mp_del_subscript() -> Option<ffi::objobjargproc> {
        None
    }
}

impl<'p, T> DeplItemDipatch for T where T: PyMappingProtocol<'p> {}

/// Returns `py_func_set_del` if PyMappingSetItemProtocol is implemented, otherwise `py_func_del`
trait DelSetItemDispatch: Sized + for<'p> PyMappingDelItemProtocol<'p> {
    fn det_set_dispatch() -> Option<ffi::objobjargproc> {
        py_func_del!(PyMappingDelItemProtocol, Self, __delitem__)
    }
}

impl<T> DelSetItemDispatch for T where T: Sized + for<'p> PyMappingDelItemProtocol<'p> {}

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
pub trait PyMappingContainsProtocolImpl {
    fn __contains__() -> Option<PyMethodDef> {
        None
    }
}

impl<'p, T> PyMappingContainsProtocolImpl for T where T: PyMappingProtocol<'p> {}

#[doc(hidden)]
pub trait PyMappingReversedProtocolImpl {
    fn __reversed__() -> Option<PyMethodDef> {
        None
    }
}

impl<'p, T> PyMappingReversedProtocolImpl for T where T: PyMappingProtocol<'p> {}

#[doc(hidden)]
pub trait PyMappingIterProtocolImpl {
    fn __iter__() -> Option<PyMethodDef> {
        None
    }
}

impl<'p, T> PyMappingIterProtocolImpl for T where T: PyMappingProtocol<'p> {}
