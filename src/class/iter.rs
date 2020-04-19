// Copyright (c) 2017-present PyO3 Project and Contributors
//! Python Iterator Interface.
//! Trait and support implementation for implementing iterators

use crate::callback::IntoPyCallbackOutput;
use crate::derive_utils::TryFromPyCell;
use crate::err::PyResult;
use crate::{ffi, IntoPy, IntoPyPointer, PyClass, PyObject, Python};

/// Python Iterator Interface.
///
/// Check [CPython doc](https://docs.python.org/3/c-api/typeobj.html#c.PyTypeObject.tp_iter)
/// for more.
#[allow(unused_variables)]
pub trait PyIterProtocol<'p>: PyClass {
    fn __iter__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyIterIterProtocol<'p>,
    {
        unimplemented!()
    }

    fn __next__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyIterNextProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyIterIterProtocol<'p>: PyIterProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Success: crate::IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyIterNextProtocol<'p>: PyIterProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Success: crate::IntoPy<PyObject>;
    type Result: Into<PyResult<Option<Self::Success>>>;
}

#[doc(hidden)]
pub trait PyIterProtocolImpl {
    fn tp_as_iter(_typeob: &mut ffi::PyTypeObject);
}

impl<T> PyIterProtocolImpl for T {
    default fn tp_as_iter(_typeob: &mut ffi::PyTypeObject) {}
}

impl<'p, T> PyIterProtocolImpl for T
where
    T: PyIterProtocol<'p>,
{
    #[inline]
    fn tp_as_iter(typeob: &mut ffi::PyTypeObject) {
        typeob.tp_iter = Self::tp_iter();
        typeob.tp_iternext = Self::tp_iternext();
    }
}

trait PyIterIterProtocolImpl {
    fn tp_iter() -> Option<ffi::getiterfunc>;
}

impl<'p, T> PyIterIterProtocolImpl for T
where
    T: PyIterProtocol<'p>,
{
    default fn tp_iter() -> Option<ffi::getiterfunc> {
        None
    }
}

impl<T> PyIterIterProtocolImpl for T
where
    T: for<'p> PyIterIterProtocol<'p>,
{
    #[inline]
    fn tp_iter() -> Option<ffi::getiterfunc> {
        py_unarys_func!(PyIterIterProtocol, T::__iter__)
    }
}

trait PyIterNextProtocolImpl {
    fn tp_iternext() -> Option<ffi::iternextfunc>;
}

impl<'p, T> PyIterNextProtocolImpl for T
where
    T: PyIterProtocol<'p>,
{
    default fn tp_iternext() -> Option<ffi::iternextfunc> {
        None
    }
}

impl<T> PyIterNextProtocolImpl for T
where
    T: for<'p> PyIterNextProtocol<'p>,
{
    #[inline]
    fn tp_iternext() -> Option<ffi::iternextfunc> {
        py_unarys_func!(PyIterNextProtocol, T::__next__, IterNextConverter)
    }
}

struct IterNextConverter<T>(Option<T>);

impl<T> IntoPyCallbackOutput<*mut ffi::PyObject> for IterNextConverter<T>
where
    T: IntoPy<PyObject>,
{
    fn convert(self, py: Python) -> PyResult<*mut ffi::PyObject> {
        match self.0 {
            Some(val) => Ok(val.into_py(py).into_ptr()),
            None => Err(crate::exceptions::StopIteration::py_err(())),
        }
    }
}
