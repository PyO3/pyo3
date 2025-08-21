# Using `async` and `await`

*This feature is still in active development. See [the related issue](https://github.com/PyO3/pyo3/issues/1632).*

`#[pyfunction]` and `#[pymethods]` attributes also support `async fn`.

```rust,no_run
# #![allow(dead_code)]
# #[cfg(feature = "experimental-async")] {
use std::{thread, time::Duration};
use futures::channel::oneshot;
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature=(seconds, result=None))]
async fn sleep(seconds: f64, result: Option<Py<PyAny>>) -> Option<Py<PyAny>> {
    let (tx, rx) = oneshot::channel();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs_f64(seconds));
        tx.send(()).unwrap();
    });
    rx.await.unwrap();
    result
}
# }
```

*Python awaitables instantiated with this method can only be awaited in *asyncio* context. Other Python async runtime may be supported in the future.*

## `Send + 'static` constraint

Resulting future of an `async fn` decorated by `#[pyfunction]` must be `Send + 'static` to be embedded in a Python object.

As a consequence, `async fn` parameters and return types must also be `Send + 'static`, so it is not possible to have a signature like `async fn does_not_compile<'py>(arg: Bound<'py, PyAny>) -> Bound<'py, PyAny>`.

However, there is an exception for method receivers, so async methods can accept `&self`/`&mut self`. Note that this means that the class instance is borrowed for as long as the returned future is not completed, even across yield points and while waiting for I/O operations to complete. Hence, other methods cannot obtain exclusive borrows while the future is still being polled. This is the same as how async methods in Rust generally work but it is more problematic for Rust code interfacing with Python code due to pervasive shared mutability. This strongly suggests to prefer shared borrows `&self` over exclusive ones `&mut self` to avoid racy borrow check failures at runtime.

## Implicitly attached to the interpreter

Even if it is not possible to pass a `py: Python<'py>` token to an `async fn`, we're still attached to the interpreter during the execution of the future – the same as for a regular `fn` without `Python<'py>`/`Bound<'py, PyAny>` parameter

It is still possible to get a `Python` marker using [`Python::attach`]({{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.attach); because `attach` is reentrant and optimized, the cost will be negligible.

## Detaching from the interpreter across `.await`

There is currently no simple way to detach from the interpreter when awaiting a future, *but solutions are currently in development*.

Here is the advised workaround for now:

```rust,ignore
use std::{
    future::Future,
    pin::{Pin, pin},
    task::{Context, Poll},
};
use pyo3::prelude::*;

struct AllowThreads<F>(F);

impl<F> Future for AllowThreads<F>
where
    F: Future + Unpin + Send,
    F::Output: Send,
{
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let waker = cx.waker();
        Python::attach(|py| {
            py.detach(|| pin!(&mut self.0).poll(&mut Context::from_waker(waker)))
        })
    }
}
```

## Cancellation

Cancellation on the Python side can be caught using [`CancelHandle`]({{#PYO3_DOCS_URL}}/pyo3/coroutine/struct.CancelHandle.html) type, by annotating a function parameter with `#[pyo3(cancel_handle)]`.

```rust,no_run
# #![allow(dead_code)]
# #[cfg(feature = "experimental-async")] {
use futures::FutureExt;
use pyo3::prelude::*;
use pyo3::coroutine::CancelHandle;

#[pyfunction]
async fn cancellable(#[pyo3(cancel_handle)] mut cancel: CancelHandle) {
    futures::select! {
        /* _ = ... => println!("done"), */
        _ = cancel.cancelled().fuse() => println!("cancelled"),
    }
}
# }
```

## The `Coroutine` type

To make a Rust future awaitable in Python, PyO3 defines a [`Coroutine`]({{#PYO3_DOCS_URL}}/pyo3/coroutine/struct.Coroutine.html) type, which implements the Python [coroutine protocol](https://docs.python.org/3/library/collections.abc.html#collections.abc.Coroutine).

Each `coroutine.send` call is translated to a `Future::poll` call. If a [`CancelHandle`]({{#PYO3_DOCS_URL}}/pyo3/coroutine/struct.CancelHandle.html) parameter is declared, the exception passed to `coroutine.throw` call is stored in it and can be retrieved with [`CancelHandle::cancelled`]({{#PYO3_DOCS_URL}}/pyo3/coroutine/struct.CancelHandle.html#method.cancelled); otherwise, it cancels the Rust future, and the exception is reraised;

*The type does not yet have a public constructor until the design is finalized.*
