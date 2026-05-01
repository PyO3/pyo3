use crate::{Py, PyAny};
use std::future::poll_fn;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

#[derive(Debug, Default)]
struct Inner {
    exception: Option<Py<PyAny>>,
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
    pub fn poll_cancelled(&mut self, cx: &mut Context<'_>) -> Poll<Py<PyAny>> {
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
    pub async fn cancelled(&mut self) -> Py<PyAny> {
        poll_fn(|cx| self.poll_cancelled(cx)).await
    }

    #[doc(hidden)]
    pub fn throw_callback(&self) -> ThrowCallback {
        ThrowCallback(self.0.clone())
    }
}

#[doc(hidden)]
pub struct ThrowCallback(Arc<Mutex<Inner>>);

impl ThrowCallback {
    pub(super) fn throw(&self, exc: Py<PyAny>) {
        let mut inner = self.0.lock().unwrap();
        inner.exception = Some(exc);
        if let Some(waker) = inner.waker.take() {
            waker.wake();
        }
    }
}
