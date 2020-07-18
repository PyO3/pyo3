// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Async/Await Interface.
//!
//! Check [the Python C API information](
//! https://docs.python.org/3/c-api/typeobj.html#async-object-structures)
//!
//! [PEP-0492](https://www.python.org/dev/peps/pep-0492/)
//!

use crate::callback::IntoPyCallbackOutput;
use crate::derive_utils::TryFromPyCell;
use crate::err::PyResult;
use crate::{ffi, IntoPy, IntoPyPointer, PyClass, PyObject, Python};

/// Python Async/Await support interface.
///
/// Each method in this trait corresponds to Python async/await implementation.
#[allow(unused_variables)]
pub trait PyAsyncProtocol<'p>: PyClass {
    fn __await__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyAsyncAwaitProtocol<'p>,
    {
        unimplemented!()
    }

    fn __aiter__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyAsyncAiterProtocol<'p>,
    {
        unimplemented!()
    }

    fn __anext__(slf: Self::Receiver) -> Self::Result
    where
        Self: PyAsyncAnextProtocol<'p>,
    {
        unimplemented!()
    }

    fn __aenter__(&'p mut self) -> Self::Result
    where
        Self: PyAsyncAenterProtocol<'p>,
    {
        unimplemented!()
    }

    fn __aexit__(
        &'p mut self,
        exc_type: Option<Self::ExcType>,
        exc_value: Option<Self::ExcValue>,
        traceback: Option<Self::Traceback>,
    ) -> Self::Result
    where
        Self: PyAsyncAexitProtocol<'p>,
    {
        unimplemented!()
    }
}

pub trait PyAsyncAwaitProtocol<'p>: PyAsyncProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyAsyncAiterProtocol<'p>: PyAsyncProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyAsyncAnextProtocol<'p>: PyAsyncProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Result: IntoPyCallbackOutput<PyIterANextOutput>;
}

pub trait PyAsyncAenterProtocol<'p>: PyAsyncProtocol<'p> {
    type Result: IntoPyCallbackOutput<PyObject>;
}

pub trait PyAsyncAexitProtocol<'p>: PyAsyncProtocol<'p> {
    type ExcType: crate::FromPyObject<'p>;
    type ExcValue: crate::FromPyObject<'p>;
    type Traceback: crate::FromPyObject<'p>;
    type Result: IntoPyCallbackOutput<PyObject>;
}

#[doc(hidden)]
impl ffi::PyAsyncMethods {
    pub fn set_await<T>(&mut self)
    where
        T: for<'p> PyAsyncAwaitProtocol<'p>,
    {
        self.am_await = py_unarys_func!(PyAsyncAwaitProtocol, T::__await__);
    }
    pub fn set_aiter<T>(&mut self)
    where
        T: for<'p> PyAsyncAiterProtocol<'p>,
    {
        self.am_aiter = py_unarys_func!(PyAsyncAiterProtocol, T::__aiter__);
    }
    pub fn set_anext<T>(&mut self)
    where
        T: for<'p> PyAsyncAnextProtocol<'p>,
    {
        self.am_anext = am_anext::<T>();
    }
}

pub enum IterANextOutput<T, U> {
    Yield(T),
    Return(U),
}

pub type PyIterANextOutput = IterANextOutput<PyObject, PyObject>;

impl IntoPyCallbackOutput<*mut ffi::PyObject> for PyIterANextOutput {
    fn convert(self, _py: Python) -> PyResult<*mut ffi::PyObject> {
        match self {
            IterANextOutput::Yield(o) => Ok(o.into_ptr()),
            IterANextOutput::Return(opt) => {
                Err(crate::exceptions::PyStopAsyncIteration::py_err((opt,)))
            }
        }
    }
}

impl<T, U> IntoPyCallbackOutput<PyIterANextOutput> for IterANextOutput<T, U>
where
    T: IntoPy<PyObject>,
    U: IntoPy<PyObject>,
{
    fn convert(self, py: Python) -> PyResult<PyIterANextOutput> {
        match self {
            IterANextOutput::Yield(o) => Ok(IterANextOutput::Yield(o.into_py(py))),
            IterANextOutput::Return(o) => Ok(IterANextOutput::Return(o.into_py(py))),
        }
    }
}

impl<T> IntoPyCallbackOutput<PyIterANextOutput> for Option<T>
where
    T: IntoPy<PyObject>,
{
    fn convert(self, py: Python) -> PyResult<PyIterANextOutput> {
        match self {
            Some(o) => Ok(PyIterANextOutput::Yield(o.into_py(py))),
            None => Ok(PyIterANextOutput::Return(py.None())),
        }
    }
}

#[inline]
fn am_anext<T>() -> Option<ffi::unaryfunc>
where
    T: for<'p> PyAsyncAnextProtocol<'p>,
{
    py_unarys_func!(PyAsyncAnextProtocol, T::__anext__)
}
