use std::{
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll, Waker},
};

use crate::PyObject;

#[derive(Debug, Default)]
struct Inner {
    exception: Option<PyObject>,
    waker: Option<Waker>,
}

/// Helper used to wait and retrieve exception thrown in [`Coroutine`](super::Coroutine).
///
/// Only the last exception thrown can be retrieved.
#[derive(Debug, Default)]
pub struct CancelHandle(Arc<Mutex<Inner>>);

impl CancelHandle {
    /// Create a new `CoroutineCancel`.
    pub fn new() -> Self {
        Default::default()
    }

    /// Returns whether the associated coroutine has been cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.0.lock().unwrap().exception.is_some()
    }

    /// Poll to retrieve the exception thrown in the associated coroutine.
    pub fn poll_cancelled(&mut self, cx: &mut Context<'_>) -> Poll<PyObject> {
        let mut inner = self.0.lock().unwrap();
        if let Some(exc) = inner.exception.take() {
            return Poll::Ready(exc);
        }
        if let Some(ref waker) = inner.waker {
            if cx.waker().will_wake(waker) {
                return Poll::Pending;
            }
        }
        inner.waker = Some(cx.waker().clone());
        Poll::Pending
    }

    /// Retrieve the exception thrown in the associated coroutine.
    pub async fn cancelled(&mut self) -> PyObject {
        // TODO use `std::future::poll_fn` with MSRV 1.64+
        struct Cancelled<'a>(&'a mut CancelHandle);

        impl Future for Cancelled<'_> {
            type Output = PyObject;
            fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
                self.0.poll_cancelled(cx)
            }
        }
        Cancelled(self).await
    }

    #[doc(hidden)]
    pub fn throw_callback(&self) -> ThrowCallback {
        ThrowCallback(self.0.clone())
    }
}

#[doc(hidden)]
pub struct ThrowCallback(Arc<Mutex<Inner>>);

impl ThrowCallback {
    pub(super) fn throw(&self, exc: PyObject) {
        let mut inner = self.0.lock().unwrap();
        inner.exception = Some(exc);
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
    }
}
