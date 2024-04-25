use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

use super::waker::try_delegate;
use crate::{
    coroutine::CoroOp,
    exceptions::{PyAttributeError, PyTypeError},
    intern,
    sync::GILOnceCell,
    types::{PyAnyMethods, PyIterator, PyTypeMethods},
    Bound, PyAny, PyErr, PyObject, PyResult, Python,
};

const NOT_IN_COROUTINE_CONTEXT: &str = "PyFuture must be awaited in coroutine context";

fn is_awaitable(obj: &Bound<'_, PyAny>) -> PyResult<bool> {
    static IS_AWAITABLE: GILOnceCell<PyObject> = GILOnceCell::new();
    let import = || {
        PyResult::Ok(
            obj.py()
                .import_bound("inspect")?
                .getattr("isawaitable")?
                .into(),
        )
    };
    IS_AWAITABLE
        .get_or_try_init(obj.py(), import)?
        .call1(obj.py(), (obj,))?
        .extract(obj.py())
}

pub(crate) enum YieldOrReturn {
    Return(PyObject),
    Yield(PyObject),
}

pub(crate) fn delegate(
    py: Python<'_>,
    await_impl: PyObject,
    op: &CoroOp,
) -> PyResult<YieldOrReturn> {
    match op {
        CoroOp::Send(obj) => {
            cfg_if::cfg_if! {
                if #[cfg(all(Py_3_10, not(PyPy), not(Py_LIMITED_API)))] {
                    let mut result = std::ptr::null_mut();
                    match unsafe { crate::ffi::PyIter_Send(await_impl.as_ptr(), obj.as_ptr(), &mut result) }
                    {
                        -1 => Err(PyErr::take(py).unwrap()),
                        0 => Ok(YieldOrReturn::Return(unsafe {
                            PyObject::from_owned_ptr(py, result)
                        })),
                        1 => Ok(YieldOrReturn::Yield(unsafe {
                            PyObject::from_owned_ptr(py, result)
                        })),
                        _ => unreachable!(),
                    }
                } else {
                    let send = intern!(py, "send");
                    if obj.is_none(py) || !await_impl.bind(py).hasattr(send).unwrap_or(false) {
                        await_impl.call_method0(py, intern!(py, "__next__"))
                    } else {
                        await_impl.call_method1(py, send, (obj,))
                    }
                    .map(YieldOrReturn::Yield)
                }
            }
        }
        CoroOp::Throw(exc) => {
            let throw = intern!(py, "throw");
            if await_impl.bind(py).hasattr(throw).unwrap_or(false) {
                await_impl
                    .call_method1(py, throw, (exc,))
                    .map(YieldOrReturn::Yield)
            } else {
                Err(PyErr::from_value_bound(exc.bind(py).clone()))
            }
        }
        CoroOp::Close => {
            let close = intern!(py, "close");
            if await_impl.bind(py).hasattr(close).unwrap_or(false) {
                await_impl
                    .call_method0(py, close)
                    .map(YieldOrReturn::Return)
            } else {
                Ok(YieldOrReturn::Return(py.None()))
            }
        }
    }
}

struct AwaitImpl(PyObject);

impl AwaitImpl {
    fn new(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        let __await__ = intern!(obj.py(), "__await__");
        match obj.call_method0(__await__) {
            Ok(iter) => Ok(Self(iter.unbind())),
            Err(err) if err.is_instance_of::<PyAttributeError>(obj.py()) => {
                if obj.hasattr(__await__)? {
                    Err(err)
                } else if is_awaitable(obj)? {
                    Ok(Self(
                        PyIterator::from_bound_object(obj)?.unbind().into_any(),
                    ))
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
}

impl Future for AwaitImpl {
    type Output = PyResult<PyObject>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match try_delegate(cx.waker(), self.0.clone()) {
            Some(poll) => poll,
            None => panic!("{}", NOT_IN_COROUTINE_CONTEXT),
        }
    }
}

/// Allows awaiting arbitrary Python awaitable inside PyO3 coroutine context, e.g. async pyfunction.
///
/// Awaiting the resulting future will panic if it's not done in coroutine context.
/// However, the future can be instantiated outside of coroutine context.
///
/// ```rust
/// use pyo3::{coroutine::await_in_coroutine, prelude::*, py_run, wrap_pyfunction_bound};
///
/// # fn main() {
/// #[pyfunction]
/// async fn wrap_awaitable(awaitable: PyObject) -> PyResult<PyObject> {
///     let future = Python::with_gil(|gil| await_in_coroutine(awaitable.bind(gil)))?;
///     future.await
/// }
/// Python::with_gil(|py| {
///     let wrap_awaitable = wrap_pyfunction_bound!(wrap_awaitable, py).unwrap();
///     let test = r#"
///         import asyncio
///         assert asyncio.run(wrap_awaitable(asyncio.sleep(1, result=42))) == 42
///      "#;
///     py_run!(py, wrap_awaitable, test);
/// })
/// # }
/// ```
/// ```rust
pub fn await_in_coroutine(
    obj: &Bound<'_, PyAny>,
) -> PyResult<impl Future<Output = PyResult<PyObject>> + Send + Sync + 'static> {
    AwaitImpl::new(obj)
}
