// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Async/Await Interface.
//!
//! Check [the Python C API information](
//! https://docs.python.org/3/c-api/typeobj.html#async-object-structures)
//!
//! [PEP-0492](https://www.python.org/dev/peps/pep-0492/)
//!

use crate::derive_utils::TryFromPyCell;
use crate::err::PyResult;
use crate::{ffi, PyClass, PyObject};

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
    type Success: crate::IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAiterProtocol<'p>: PyAsyncProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Success: crate::IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAnextProtocol<'p>: PyAsyncProtocol<'p> {
    type Receiver: TryFromPyCell<'p, Self>;
    type Success: crate::IntoPy<PyObject>;
    type Result: Into<PyResult<Option<Self::Success>>>;
}

pub trait PyAsyncAenterProtocol<'p>: PyAsyncProtocol<'p> {
    type Success: crate::IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAexitProtocol<'p>: PyAsyncProtocol<'p> {
    type ExcType: crate::FromPyObject<'p>;
    type ExcValue: crate::FromPyObject<'p>;
    type Traceback: crate::FromPyObject<'p>;
    type Success: crate::IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

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
        self.am_anext = anext::am_anext::<T>();
    }
}

mod anext {
    use super::PyAsyncAnextProtocol;
    use crate::callback::IntoPyCallbackOutput;
    use crate::err::PyResult;
    use crate::IntoPyPointer;
    use crate::Python;
    use crate::{ffi, IntoPy, PyObject};

    struct IterANextOutput<T>(Option<T>);

    impl<T> IntoPyCallbackOutput<*mut ffi::PyObject> for IterANextOutput<T>
    where
        T: IntoPy<PyObject>,
    {
        fn convert(self, py: Python) -> PyResult<*mut ffi::PyObject> {
            match self.0 {
                Some(val) => Ok(val.into_py(py).into_ptr()),
                None => Err(crate::exceptions::StopAsyncIteration::py_err(())),
            }
        }
    }

    #[inline]
    pub(super) fn am_anext<T>() -> Option<ffi::unaryfunc>
    where
        T: for<'p> PyAsyncAnextProtocol<'p>,
    {
        py_unarys_func!(PyAsyncAnextProtocol, T::__anext__, IterANextOutput)
    }
}
