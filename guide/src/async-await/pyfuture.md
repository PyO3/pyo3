# Awaiting Python awaitables

Python awaitable can be awaited on Rust side using [`PyFuture`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyFuture.html).

```rust
# #![allow(dead_code)]
use pyo3::{prelude::*, types::PyFuture};

#[pyfunction]
async fn wrap_awaitable(awaitable: PyObject) -> PyResult<PyObject> {
    let future: Py<PyFuture> = Python::with_gil(|gil| Py::from_object(gil, awaitable))?;
    future.await
}
```

`PyFuture::from_object` construct a `PyFuture` from a Python awaitable object, by calling its `__await__` method (or `__iter__` for generator-based coroutine).

## Restrictions

`PyFuture` can only be awaited in the context of a PyO3 coroutine. Otherwise, it panics.

```rust
# #![allow(dead_code)]
use pyo3::{prelude::*, types::PyFuture};

#[pyfunction]
fn block_on(awaitable: PyObject) -> PyResult<PyObject> {
    let future: Py<PyFuture> = Python::with_gil(|gil| Py::from_object(gil, awaitable))?;
    futures::executor::block_on(future) // ERROR: PyFuture must be awaited in coroutine context
}
```

`PyFuture` must be the only Rust future awaited; it means that it's forbidden to `select!` a `Pyfuture`. Otherwise, it panics.

```rust
# #![allow(dead_code)]
use std::future;
use futures::FutureExt;
use pyo3::{prelude::*, types::PyFuture};

#[pyfunction]
async fn select(awaitable: PyObject) -> PyResult<PyObject> {
    let future: Py<PyFuture> = Python::with_gil(|gil| Py::from_object(gil, awaitable))?;
    futures::select_biased! {
        _ = future::pending::<()>().fuse() => unreachable!(),
        res = future.fuse() => res, // ERROR: Python awaitable mixed with Rust future
    }
}
```

These restrictions exist because awaiting a `PyFuture` strongly binds it to the enclosing coroutine. The coroutine will then delegate its `send`/`throw`/`close` methods to the awaited `PyFuture`. If it was awaited in a `select!`, `Coroutine::send` would no able to know if the value passed would have to be delegated to the `Pyfuture` or not.
