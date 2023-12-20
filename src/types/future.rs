use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[cfg(feature = "macros")]
use crate::{
    coroutine::waker::{FutureOrPoll, FUTURE_OR_POLL},
    pyclass::IterNextOutput,
    PyErr,
};
use crate::{
    exceptions::{PyAttributeError, PyTypeError},
    ffi,
    instance::Py2,
    sync::GILOnceCell,
    types::any::PyAnyMethods,
    IntoPy, Py, PyAny, PyNativeType, PyObject, PyResult, Python,
};

const NOT_IN_COROUTINE_CONTEXT: &str = "PyFuture must be awaited in coroutine context";

/// A Python object returned by `__await__`.
///
/// # Examples
///
/// ```rust
/// use pyo3::prelude::*;
///
/// # fn main() -> PyResult<()> {
/// Python::with_gil(|py| -> PyResult<()> {
///     let list = py.eval("iter([1, 2, 3, 4])", None, None)?;
///     let numbers: PyResult<Vec<usize>> = list
///         .iter()?
///         .map(|i| i.and_then(PyAny::extract::<usize>))
///         .collect();
///     let sum: usize = numbers?.iter().sum();
///     assert_eq!(sum, 10);
///     Ok(())
/// })
/// # }
/// ```
#[repr(transparent)]
pub struct PyFuture(PyAny);
pyobject_native_type_named!(PyFuture);

fn is_awaitable(obj: &Py2<'_, PyAny>) -> PyResult<bool> {
    static IS_AWAITABLE: GILOnceCell<PyObject> = GILOnceCell::new();
    let import = || PyResult::Ok(obj.py().import("inspect")?.getattr("isawaitable")?.into());
    IS_AWAITABLE
        .get_or_try_init(obj.py(), import)?
        .call1(obj.py(), (obj,))?
        .extract(obj.py())
}

impl PyFuture {
    /// Constructs a `PyFuture` from a Python awaitable object.
    ///
    /// Equivalent to calling `__await__` method (or `__iter__` for generator-based coroutines).
    pub fn from_object(obj: &PyAny) -> PyResult<&PyFuture> {
        Self::from_object2(Py2::borrowed_from_gil_ref(&obj)).map(|py2| {
            // Can't use into_gil_ref here because T: PyTypeInfo bound is not satisfied
            // Safety: into_ptr produces a valid pointer to PyFuture object
            unsafe { obj.py().from_owned_ptr(py2.into_ptr()) }
        })
    }

    pub(crate) fn from_object2<'py>(obj: &Py2<'py, PyAny>) -> PyResult<Py2<'py, PyFuture>> {
        let __await__ = intern!(obj.py(), "__await__");
        match obj.call_method0(__await__) {
            Ok(obj) => Ok(unsafe { obj.downcast_into_unchecked() }),
            Err(err) if err.is_instance_of::<PyAttributeError>(obj.py()) => {
                if obj.hasattr(__await__)? {
                    Err(err)
                } else if is_awaitable(obj)? {
                    unsafe {
                        Py2::from_owned_ptr_or_err(obj.py(), ffi::PyObject_GetIter(obj.as_ptr()))
                            .map(|any| any.downcast_into_unchecked())
                    }
                } else {
                    Err(PyTypeError::new_err(format!(
                        "object {tp} can't be used in 'await' expression",
                        tp = obj.get_type().name()?
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    #[cfg(feature = "macros")]
    pub(crate) fn next(
        &self,
        prev_result: &Option<Result<PyObject, PyObject>>,
    ) -> PyResult<IterNextOutput<PyObject, PyObject>> {
        let py = self.0.py();
        match prev_result {
            Some(Ok(val)) => {
                cfg_if::cfg_if! {
                    if #[cfg(all(Py_3_10, not(PyPy), not(Py_LIMITED_API)))] {
                        let mut result = std::ptr::null_mut();
                        match unsafe { ffi::PyIter_Send(self.0.as_ptr(), val.as_ptr(), &mut result) }
                        {
                            -1 => Err(PyErr::take(py).unwrap()),
                            0 => Ok(IterNextOutput::Return(unsafe {
                                PyObject::from_owned_ptr(py, result)
                            })),
                            1 => Ok(IterNextOutput::Yield(unsafe {
                                PyObject::from_owned_ptr(py, result)
                            })),
                            _ => unreachable!(),
                        }
                    } else {
                        let send  = intern!(py, "send");
                        if val.is_none(py) || !self.0.hasattr(send).unwrap_or(false) {
                            self.0.call_method0(intern!(py, "__next__"))
                        } else {
                            self.0.call_method1(send, (val,))
                        }
                        .map(Into::into)
                        .map(IterNextOutput::Yield)
                    }
                }
            }
            Some(Err(err)) => {
                let throw = intern!(py, "throw");
                if self.0.hasattr(throw).unwrap_or(false) {
                    self.0
                        .call_method1(throw, (err,))
                        .map(Into::into)
                        .map(IterNextOutput::Yield)
                } else {
                    Err(PyErr::from_value(err.as_ref(py)))
                }
            }
            None => {
                let close = intern!(py, "close");
                if self.0.hasattr(close).unwrap_or(false) {
                    self.0
                        .call_method0(close)
                        .map(Into::into)
                        .map(IterNextOutput::Return)
                } else {
                    Ok(IterNextOutput::Return(py.None().into()))
                }
            }
        }
    }
}

impl Future for Py<PyFuture> {
    type Output = PyResult<PyObject>;

    #[cfg(feature = "macros")]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        FUTURE_OR_POLL.with(|cell| cell.set(Some(FutureOrPoll::Future(self.clone()))));
        cx.waker().wake_by_ref();
        match FUTURE_OR_POLL.with(|cell| cell.take()) {
            Some(FutureOrPoll::Poll(poll)) => poll,
            Some(FutureOrPoll::Future(_)) => panic!("{}", NOT_IN_COROUTINE_CONTEXT),
            None => unreachable!(),
        }
    }
    #[cfg(not(feature = "macros"))]
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        panic!("{}", NOT_IN_COROUTINE_CONTEXT)
    }
}

impl Py<PyFuture> {
    /// Constructs a `PyFuture` from a Python awaitable object.
    ///
    /// Equivalent to calling `__await__` method (or `__iter__` for generator-based coroutines).
    pub fn from_object(py: Python<'_>, awaitable: PyObject) -> PyResult<Self> {
        Ok(PyFuture::from_object(awaitable.as_ref(py))?.into_py(py))
    }
}
