// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Async/Await Interface.
//!
//! Check [the Python C API information](
//! https://docs.python.org/3/c-api/typeobj.html#async-object-structures)
//!
//! [PEP-0492](https://www.python.org/dev/peps/pep-0492/)
//!

use crate::err::PyResult;
use crate::{ffi, Py, PyAny, PyClass};

/// Python Async/Await support interface.
///
/// Each method in this trait corresponds to Python async/await implementation.
#[allow(unused_variables)]
pub trait PyAsyncProtocol<'p>: PyClass {
    fn __await__(&'p self) -> Self::Result
    where
        Self: PyAsyncAwaitProtocol<'p>,
    {
        unimplemented!()
    }

    fn __aiter__(&'p self) -> Self::Result
    where
        Self: PyAsyncAiterProtocol<'p>,
    {
        unimplemented!()
    }

    fn __anext__(&'p mut self) -> Self::Result
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
    type Success: crate::IntoPy<Py<PyAny>>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAiterProtocol<'p>: PyAsyncProtocol<'p> {
    type Success: crate::IntoPy<Py<PyAny>>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAnextProtocol<'p>: PyAsyncProtocol<'p> {
    type Success: crate::IntoPy<Py<PyAny>>;
    type Result: Into<PyResult<Option<Self::Success>>>;
}

pub trait PyAsyncAenterProtocol<'p>: PyAsyncProtocol<'p> {
    type Success: crate::IntoPy<Py<PyAny>>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAexitProtocol<'p>: PyAsyncProtocol<'p> {
    type ExcType: crate::FromPyObject<'p>;
    type ExcValue: crate::FromPyObject<'p>;
    type Traceback: crate::FromPyObject<'p>;
    type Success: crate::IntoPy<Py<PyAny>>;
    type Result: Into<PyResult<Self::Success>>;
}

#[doc(hidden)]
pub trait PyAsyncProtocolImpl {
    fn tp_as_async() -> Option<ffi::PyAsyncMethods>;
}

impl<T> PyAsyncProtocolImpl for T {
    default fn tp_as_async() -> Option<ffi::PyAsyncMethods> {
        None
    }
}

impl<'p, T> PyAsyncProtocolImpl for T
where
    T: PyAsyncProtocol<'p>,
{
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

impl<'p, T> PyAsyncAwaitProtocolImpl for T
where
    T: PyAsyncProtocol<'p>,
{
    default fn am_await() -> Option<ffi::unaryfunc> {
        None
    }
}

impl<T> PyAsyncAwaitProtocolImpl for T
where
    T: for<'p> PyAsyncAwaitProtocol<'p>,
{
    #[inline]
    fn am_await() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyAsyncAwaitProtocol, T::__await__)
    }
}

trait PyAsyncAiterProtocolImpl {
    fn am_aiter() -> Option<ffi::unaryfunc>;
}

impl<'p, T> PyAsyncAiterProtocolImpl for T
where
    T: PyAsyncProtocol<'p>,
{
    default fn am_aiter() -> Option<ffi::unaryfunc> {
        None
    }
}

impl<T> PyAsyncAiterProtocolImpl for T
where
    T: for<'p> PyAsyncAiterProtocol<'p>,
{
    #[inline]
    fn am_aiter() -> Option<ffi::unaryfunc> {
        py_unary_func!(PyAsyncAiterProtocol, T::__aiter__)
    }
}

trait PyAsyncAnextProtocolImpl {
    fn am_anext() -> Option<ffi::unaryfunc>;
}

impl<'p, T> PyAsyncAnextProtocolImpl for T
where
    T: PyAsyncProtocol<'p>,
{
    default fn am_anext() -> Option<ffi::unaryfunc> {
        None
    }
}

mod anext {
    use super::{PyAsyncAnextProtocol, PyAsyncAnextProtocolImpl};
    use crate::callback::IntoPyCallbackOutput;
    use crate::err::PyResult;
    use crate::Python;
    use crate::{ffi, IntoPy, IntoPyPointer, Py, PyAny};

    struct IterANextOutput<T>(Option<T>);

    impl<T> IntoPyCallbackOutput<*mut ffi::PyObject> for IterANextOutput<T>
    where
        T: IntoPy<Py<PyAny>>,
    {
        fn convert(self, py: Python) -> PyResult<*mut ffi::PyObject> {
            match self.0 {
                Some(val) => Ok(val.into_py(py).into_ptr()),
                None => Err(crate::exceptions::StopAsyncIteration::py_err(())),
            }
        }
    }

    impl<T> PyAsyncAnextProtocolImpl for T
    where
        T: for<'p> PyAsyncAnextProtocol<'p>,
    {
        #[inline]
        fn am_anext() -> Option<ffi::unaryfunc> {
            py_unary_func!(
                PyAsyncAnextProtocol,
                T::__anext__,
                call_mut,
                *mut crate::ffi::PyObject,
                IterANextOutput
            )
        }
    }
}
