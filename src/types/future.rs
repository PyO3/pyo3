use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll, Waker},
};

use parking_lot::Mutex;

use crate::types::PyCFunction;
use crate::{
    pyobject_native_type_named,
    sync::GILOnceCell,
    types::{any::PyAnyMethods, tuple::PyTupleMethods},
    Bound, Py, PyAny, PyObject, PyResult, PyTypeCheck, Python,
};

/// A Python future-like object.
///
/// It can be either an asyncio future-like or a concurrent.futures.Future object.
#[repr(transparent)]
pub struct PyFuture(PyAny);
pyobject_native_type_named!(PyFuture);

impl Py<PyFuture> {
    /// Convert a `PyFuture` into a Rust `Future`.
    ///
    /// Contrary to Python futures, Rust future will panic if polled after completion,
    /// to allow some optimizations.
    pub fn as_rust_future(
        &self,
        py: Python<'_>,
    ) -> PyResult<impl Future<Output = PyResult<PyObject>> + Send + Sync + 'static> {
        self.bind(py).as_rust_future()
    }
}

impl Bound<'_, PyFuture> {
    /// Convert a `PyFuture` into a Rust `Future`.
    ///
    /// Contrary to Python futures, Rust future will panic if polled after completion,
    /// to allow some optimizations.
    pub fn as_rust_future(
        &self,
    ) -> PyResult<impl Future<Output = PyResult<PyObject>> + Send + Sync + 'static> {
        #[derive(Default)]
        struct PendingInner {
            result: Option<PyResult<PyObject>>,
            waker: Option<Waker>,
        }
        enum FutureImpl {
            Done(Option<PyResult<PyObject>>),
            Pending(Arc<Mutex<PendingInner>>),
        }
        impl Future for FutureImpl {
            type Output = PyResult<PyObject>;
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                let this = self.get_mut();
                match this {
                    Self::Done(res) => {
                        // The future panics if already completed to avoid cloning the result,
                        // which would require acquiring the GIL or the internal pool.
                        Poll::Ready(res.take().expect("Future polled after completion"))
                    }
                    Self::Pending(cb) => {
                        let mut inner = cb.lock();
                        if inner.result.is_some() {
                            let res = inner.result.take().unwrap();
                            drop(inner);
                            *this = Self::Done(None);
                            return Poll::Ready(res);
                        }
                        if !matches!(&mut inner.waker, Some(waker) if waker.will_wake(cx.waker())) {
                            inner.waker = Some(cx.waker().clone());
                        }
                        Poll::Pending
                    }
                }
            }
        }
        Ok(if self.call_method0("done")?.extract()? {
            FutureImpl::Done(Some(self.call_method0("result").map(Bound::unbind)))
        } else {
            let pending = Arc::new(Mutex::new(PendingInner::default()));
            let pending2 = pending.clone();
            let callback =
                PyCFunction::new_closure_bound(self.py(), None, None, move |args, _| {
                    let result = args.get_item(0)?.call_method0("result").map(Bound::unbind);
                    let mut inner = pending2.lock();
                    inner.result = Some(result);
                    if let Some(waker) = &inner.waker {
                        waker.wake_by_ref();
                    }
                    PyResult::Ok(())
                })?;
            self.call_method1("add_done_callback", (callback,))?;
            // For asyncio futures, `add_done_callback` should be called in the event loop thread
            // because it calls `loop.call_soon` if the future is done. By calling a second time
            // `Future.done`, we ensure that the whole operation is thread safe: if `loop.call_soon`
            // is called from the wrong thread, `Future.done` will then return true, and the
            // callback (which could have not been executed because loop may be sleeping)
            // will not be used.
            if self.call_method0("done")?.extract()? {
                FutureImpl::Done(Some(self.call_method0("result").map(Bound::unbind)))
            } else {
                FutureImpl::Pending(pending)
            }
        })
    }
}

fn is_asyncio_future(object: &Bound<'_, PyAny>) -> PyResult<bool> {
    static IS_FUTURE: GILOnceCell<PyObject> = GILOnceCell::new();
    let import = || {
        let module = object.py().import_bound("asyncio")?;
        PyResult::Ok(module.getattr("isfuture")?.into())
    };
    IS_FUTURE
        .get_or_try_init(object.py(), import)?
        .call1(object.py(), (object,))?
        .extract(object.py())
}

fn is_concurrent_future(object: &Bound<'_, PyAny>) -> PyResult<bool> {
    static FUTURE: GILOnceCell<PyObject> = GILOnceCell::new();
    let import = || {
        let module = object.py().import_bound("concurrent.futures")?;
        PyResult::Ok(module.getattr("Future")?.into())
    };
    let future_type = FUTURE
        .get_or_try_init(object.py(), import)?
        .bind(object.py());
    object.is_instance(future_type)
}

impl PyTypeCheck for PyFuture {
    const NAME: &'static str = "Future";

    fn type_check(object: &Bound<'_, PyAny>) -> bool {
        is_asyncio_future(object).unwrap_or(false) || is_concurrent_future(object).unwrap_or(false)
    }
}
