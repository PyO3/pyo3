// Copyright (c) 2017-present PyO3 Project and Contributors

//! Python Async/Await Interface.
//!
//! Check [python c-api information](
//! https://docs.python.org/3/c-api/typeobj.html#async-object-structures)
//!
//! [PEP-0492](https://www.python.org/dev/peps/pep-0492/)
//!

use crate::callback::PyObjectCallbackConverter;
use crate::class::methods::PyMethodDef;
use crate::err::PyResult;
use crate::{ffi, PyClass, PyObject};

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
    type Success: crate::IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAiterProtocol<'p>: PyAsyncProtocol<'p> {
    type Success: crate::IntoPy<PyObject>;
    type Result: Into<PyResult<Self::Success>>;
}

pub trait PyAsyncAnextProtocol<'p>: PyAsyncProtocol<'p> {
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

#[doc(hidden)]
pub trait PyAsyncProtocolImpl {
    fn tp_as_async() -> Option<ffi::PyAsyncMethods>;
    fn methods() -> Vec<PyMethodDef>;
}

impl<T> PyAsyncProtocolImpl for T {
    default fn tp_as_async() -> Option<ffi::PyAsyncMethods> {
        None
    }

    default fn methods() -> Vec<PyMethodDef> {
        Vec::new()
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

    #[inline]
    fn methods() -> Vec<PyMethodDef> {
        let mut methods = Vec::new();

        if let Some(def) = <Self as PyAsyncAenterProtocolImpl>::__aenter__() {
            methods.push(def)
        }
        if let Some(def) = <Self as PyAsyncAexitProtocolImpl>::__aexit__() {
            methods.push(def)
        }

        methods
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
        py_unary_func!(
            PyAsyncAwaitProtocol,
            T::__await__,
            PyObjectCallbackConverter::<T::Success>(std::marker::PhantomData)
        )
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
        py_unary_func!(
            PyAsyncAiterProtocol,
            T::__aiter__,
            PyObjectCallbackConverter::<T::Success>(std::marker::PhantomData)
        )
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
    use crate::callback::CallbackConverter;
    use crate::IntoPyPointer;
    use crate::Python;
    use crate::{ffi, IntoPy, PyObject};
    use std::marker::PhantomData;
    use std::ptr;

    struct IterANextResultConverter<T>(PhantomData<T>);

    impl<T> CallbackConverter for IterANextResultConverter<T>
    where
        T: IntoPy<PyObject>,
    {
        type Source = Option<T>;
        type Result = *mut ffi::PyObject;
        const ERR_VALUE: Self::Result = ptr::null_mut();

        fn convert(val: Self::Source, py: Python) -> Self::Result {
            match val {
                Some(val) => val.into_py(py).into_ptr(),
                None => unsafe {
                    ffi::PyErr_SetNone(ffi::PyExc_StopAsyncIteration);
                    ptr::null_mut()
                },
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
                IterANextResultConverter::<T::Success>(std::marker::PhantomData),
                *mut crate::ffi::PyObject,
                call_mut_with_converter
            )
        }
    }
}

trait PyAsyncAenterProtocolImpl {
    fn __aenter__() -> Option<PyMethodDef>;
}

impl<'p, T> PyAsyncAenterProtocolImpl for T
where
    T: PyAsyncProtocol<'p>,
{
    default fn __aenter__() -> Option<PyMethodDef> {
        None
    }
}

trait PyAsyncAexitProtocolImpl {
    fn __aexit__() -> Option<PyMethodDef>;
}

impl<'p, T> PyAsyncAexitProtocolImpl for T
where
    T: PyAsyncProtocol<'p>,
{
    default fn __aexit__() -> Option<PyMethodDef> {
        None
    }
}
