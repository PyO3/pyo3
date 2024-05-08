# Awaiting Python awaitables

Python awaitable can be awaited on Rust side
using [`await_in_coroutine`]({{#PYO3_DOCS_URL}}/pyo3/coroutine/function.await_in_coroutine).

```rust
# # ![allow(dead_code)]
# #[cfg(feature = "experimental-async")] {
use pyo3::{prelude::*, coroutine::await_in_coroutine};

#[pyfunction]
async fn wrap_awaitable(awaitable: PyObject) -> PyResult<PyObject> {
    Python::with_gil(|gil| await_in_coroutine(awaitable.bind(gil)))?.await
}
# }
```

Behind the scene, `await_in_coroutine` calls the `__await__` method of the Python awaitable (or `__iter__` for
generator-based coroutine).

## Restrictions

As the name suggests, `await_in_coroutine` resulting future can only be awaited in coroutine context. Otherwise, it
panics.

```rust
# # ![allow(dead_code)]
# #[cfg(feature = "experimental-async")] {
use pyo3::{prelude::*, coroutine::await_in_coroutine};

#[pyfunction]
fn block_on(awaitable: PyObject) -> PyResult<PyObject> {
    let future = Python::with_gil(|gil| await_in_coroutine(awaitable.bind(gil)))?;
    futures::executor::block_on(future) // ERROR: PyFuture must be awaited in coroutine context
}
# }
```

The future must also be the only one to be awaited at a time; it means that it's forbidden to await it in a `select!`.
Otherwise, it panics.

```rust
# # ![allow(dead_code)]
# #[cfg(feature = "experimental-async")] {
use futures::FutureExt;
use pyo3::{prelude::*, coroutine::await_in_coroutine};

#[pyfunction]
async fn select(awaitable: PyObject) -> PyResult<PyObject> {
    let future = Python::with_gil(|gil| await_in_coroutine(awaitable.bind(gil)))?;
    futures::select_biased! {
        _ = std::future::pending::<()>().fuse() => unreachable!(),
        res = future.fuse() => res, // ERROR: Python awaitable mixed with Rust future
    }
}
# }
```

These restrictions exist because awaiting a `await_in_coroutine` future strongly binds it to the
enclosing coroutine. The coroutine will then delegate its `send`/`throw`/`close` methods to the
awaited future. If it was awaited in a `select!`, `Coroutine::send` would no able to know if
the value passed would have to be delegated or not.
