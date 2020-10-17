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

#[cfg(Py_LIMITED_API)]
#[derive(Clone, Default)]
pub struct PyAsyncMethods {
    pub am_await: Option<ffi::unaryfunc>,
    pub am_aiter: Option<ffi::unaryfunc>,
    pub am_anext: Option<ffi::unaryfunc>,
}

#[cfg(not(Py_LIMITED_API))]
pub use ffi::PyAsyncMethods;

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

/// Extension trait for proc-macro backend.
#[doc(hidden)]
pub trait PyAsyncSlots {
    fn get_await() -> ffi::PyType_Slot
    where
        Self: for<'p> PyAsyncAwaitProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_am_await,
            pfunc: py_unarys_func!(PyAsyncAwaitProtocol, Self::__await__) as _,
        }
    }

    fn get_aiter() -> ffi::PyType_Slot
    where
        Self: for<'p> PyAsyncAiterProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_am_await,
            pfunc: py_unarys_func!(PyAsyncAiterProtocol, Self::__aiter__) as _,
        }
    }

    fn get_anext() -> ffi::PyType_Slot
    where
        Self: for<'p> PyAsyncAnextProtocol<'p>,
    {
        ffi::PyType_Slot {
            slot: ffi::Py_am_anext,
            pfunc: py_unarys_func!(PyAsyncAnextProtocol, Self::__anext__) as _,
        }
    }
}

impl<'p, T> PyAsyncSlots for T where T: PyAsyncProtocol<'p> {}

/// Output of `__anext__`.
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
                Err(crate::exceptions::PyStopAsyncIteration::new_err((opt,)))
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
