# Async / Await

If you are working with a Python library that makes use of async functions or wish to provide 
Python bindings for an async Rust library, [`pyo3-asyncio`](https://github.com/awestlake87/pyo3-asyncio)
likely has the tools you need. It provides conversions between async functions in both Python and 
Rust and was designed with first-class support for popular Rust runtimes such as 
[`tokio`](https://tokio.rs/) and [`async-std`](https://async.rs/). In addition, all async Python 
code runs on the default `asyncio` event loop, so `pyo3-asyncio` should work just fine with existing 
Python libraries.

In the following sections, we'll give a general overview of `pyo3-asyncio` explaining how to call 
async Python functions with PyO3, how to call async Rust functions from Python, and how to configure
your codebase to manage the runtimes of both.

## Awaiting an Async Python Function in Rust

Let's take a look at a dead simple async Python function:

```python
# Sleep for 1 second
async def py_sleep():
    await asyncio.sleep(1)
```

**Async functions in Python are simply functions that return a `coroutine` object**. For our purposes, 
we really don't need to know much about these `coroutine` objects. The key factor here is that calling
an `async` function is _just like calling a regular function_, the only difference is that we have
to do something special with the object that it returns.

Normally in Python, that something special is the `await` keyword, but in order to await this 
coroutine in Rust, we first need to convert it into Rust's version of a `coroutine`: a `Future`. 
That's where `pyo3-asyncio` comes in. 
[`pyo3_asyncio::into_future`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/fn.into_future.html) 
performs this conversion for us:


```rust
let future = Python::with_gil(|py| {
    // import the module containing the py_sleep function
    let example = py.import("example")?;

    // calling the py_sleep method like a normal function returns a coroutine
    let coroutine = example.call_method0("py_sleep")?;

    // convert the coroutine into a Rust future
    pyo3_asyncio::into_future(coroutine)
})?;

// await the future
future.await;
```

> If you're interested in learning more about `coroutines` and `awaitables` in general, check out the 
> [Python 3 `asyncio` docs](https://docs.python.org/3/library/asyncio-task.html) for more information.

## Awaiting a Rust Future in Python

Here we have the same async function as before written in Rust using the 
[`async-std`](https://async.rs/) runtime:

```rust
/// Sleep for 1 second
async fn rust_sleep() {
    async_std::task::sleep(Duration::from_secs(1)).await;
}
```

Similar to Python, Rust's async functions also return a special object called a
`Future`:

```rust
let future = rust_sleep();
```

We can convert this `Future` object into Python to make it `awaitable`. This tells Python that you 
can use the `await` keyword with it. In order to do this, we'll call 
[`pyo3_asyncio::async_std::into_coroutine`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/async_std/fn.into_coroutine.html):

```rust
#[pyfunction]
fn call_rust_sleep(py: Python) -> PyResult<PyObject> {
    pyo3_asyncio::async_std::into_coroutine(py, async move {
        rust_sleep().await;
        Ok(())
    })
}
```

In Python, we can call this pyo3 function just like any other async function:

```python
from example import call_rust_sleep

async def rust_sleep():
    await call_rust_sleep()
```

## Managing Event Loops

Python's event loop requires some special treatment, especially regarding the main thread. Some of
Python's `asyncio` features, like proper signal handling, require control over the main thread, which
doesn't always play well with Rust.

Luckily, Rust's event loops are pretty flexible and don't _need_ control over the main thread, so in
`pyo3-asyncio`, we decided the best way to handle Rust/Python interop was to just surrender the main
thread to Python and run Rust's event loops in the background. Unfortunately, since most event loop 
implementations _prefer_ control over the main thread, this can still make some things awkward.

### PyO3 Asyncio Initialization

Because Python needs to control the main thread, we can't use the convenient proc macros from Rust
runtimes to handle the `main` function or `#[test]` functions. Instead, the initialization for PyO3 has to be done from the `main` function and the main 
thread must block on [`pyo3_asyncio::run_forever`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/fn.run_forever.html) or [`pyo3_asyncio::async_std::run_until_complete`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/async_std/fn.run_until_complete.html).

Because we have to block on one of those functions, we can't use [`#[async_std::main]`](https://docs.rs/async-std/latest/async_std/attr.main.html) or [`#[tokio::main]`](https://docs.rs/tokio/1.1.0/tokio/attr.main.html)
since it's not a good idea to make long blocking calls during an async function.

> Internally, these `#[main]` proc macros are expanded to something like this:
> ```rust
> fn main() {
>     // your async main fn
>     async fn _main_impl() { /* ... */ }
>     Runtime::new().block_on(_main_impl());   
> }
> ```
> Making a long blocking call inside the `Future` that's being driven by `block_on` prevents that
> thread from doing anything else and can spell trouble for some runtimes (also this will actually 
> deadlock a single-threaded runtime!). Many runtimes have some sort of `spawn_blocking` mechanism 
> that can avoid this problem, but again that's not something we can use here since we need it to 
> block on the _main_ thread.

For this reason, `pyo3-asyncio` provides its own set of proc macros to provide you with this 
initialization. These macros are intended to mirror the initialization of `async-std` and `tokio` 
while also satisfying the Python runtime's needs.

Here's a full example of PyO3 initialization with the `async-std` runtime:
```rust
use pyo3::prelude::*;

#[pyo3_asyncio::async_std::main]
async fn main() -> PyResult<()> {
    // PyO3 is initialized - Ready to go

    let fut = Python::with_gil(|py| {
        let asyncio = py.import("asyncio")?;

        // convert asyncio.sleep into a Rust Future
        pyo3_asyncio::into_future(asyncio.call_method1("sleep", (1.into_py(py),))?)
    })?;

    fut.await?;

    Ok(())
}
```

## PyO3 Asyncio in Cargo Tests

The default Cargo Test harness does not currently allow test crates to provide their own `main` 
function, so there doesn't seem to be a good way to allow Python to gain control over the main
thread.

We can, however, override the default test harness and provide our own. `pyo3-asyncio` provides some
utilities to help us do just that! In the following sections, we will provide an overview for 
constructing a Cargo integration test with `pyo3-asyncio` and adding your tests to it.

### Main Test File
First, we need to create the test's main file. Although these tests are considered integration
tests, we cannot put them in the `tests` directory since that is a special directory owned by
Cargo. Instead, we put our tests in a `pytests` directory.

> The name `pytests` is just a convention. You can name this folder anything you want in your own
> projects.

We'll also want to provide the test's main function. Most of the functionality that the test harness needs is packed in the [`pyo3_asyncio::testing::main`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/testing/fn.main.html) function. This function will parse the test's CLI arguments, collect and pass the functions marked with [`#[pyo3_asyncio::async_std::test]`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/async_std/attr.test.html) or [`#[pyo3_asyncio::tokio::test]`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/tokio/attr.test.html) and pass them into the test harness for running and filtering.

`pytests/test_example.rs` for the `tokio` runtime:
```rust
#[pyo3_asyncio::tokio::main]
async fn main() -> pyo3::PyResult<()> {
    pyo3_asyncio::testing::main().await
}
```

`pytests/test_example.rs` for the `async-std` runtime:
```rust
#[pyo3_asyncio::async_std::main]
async fn main() -> pyo3::PyResult<()> {
    pyo3_asyncio::testing::main().await
}
```

### Cargo Configuration
Next, we need to add our test file to the Cargo manifest by adding the following section to the
`Cargo.toml`

```toml
[[test]]
name = "test_example"
path = "pytests/test_example.rs"
harness = false
```

Also add the `testing` and `attributes` features to the `pyo3-asyncio` dependency and select your preferred runtime:

```toml
pyo3-asyncio = { version = "0.13", features = ["testing", "attributes", "async-std-runtime"] }
```

At this point, you should be able to run the test via `cargo test`

### Adding Tests to the PyO3 Asyncio Test Harness

We can add tests anywhere in the test crate with the runtime's corresponding `#[test]` attribute:

For `async-std` use the [`pyo3_asyncio::async_std::test`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/async_std/attr.test.html) attribute:
```rust
mod tests {
    use std::{time::Duration, thread};

    use pyo3::prelude::*;

    // tests can be async
    #[pyo3_asyncio::async_std::test]
    async fn test_async_sleep() -> PyResult<()> {
        async_std::task::sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    // they can also be synchronous
    #[pyo3_asyncio::async_std::test]
    fn test_blocking_sleep() -> PyResult<()> {
        thread::sleep(Duration::from_secs(1));
        Ok(())
    }
}

#[pyo3_asyncio::async_std::main]
async fn main() -> pyo3::PyResult<()> {
    pyo3_asyncio::testing::main().await
}
```

For `tokio` use the [`pyo3_asyncio::tokio::test`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/tokio/attr.test.html) attribute:
```rust
mod tests {
    use std::{time::Duration, thread};

    use pyo3::prelude::*;

    // tests can be async
    #[pyo3_asyncio::tokio::test]
    async fn test_async_sleep() -> PyResult<()> {
        tokio::time::sleep(Duration::from_secs(1)).await;
        Ok(())
    }

    // they can also be synchronous
    #[pyo3_asyncio::tokio::test]
    fn test_blocking_sleep() -> PyResult<()> {
        thread::sleep(Duration::from_secs(1));
        Ok(())
    }
}

#[pyo3_asyncio::tokio::main]
async fn main() -> pyo3::PyResult<()> {
    pyo3_asyncio::testing::main().await
}
```
