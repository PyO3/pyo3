# Using `async` and `await`

*`async`/`await` support is currently being integrated in PyO3. See the [dedicated documentation](../async-await.md)*

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

## Quickstart

Here are some examples to get you started right away! A more detailed breakdown
of the concepts in these examples can be found in the following sections.

### Rust Applications
Here we initialize the runtime, import Python's `asyncio` library and run the given future to completion using Python's default `EventLoop` and `async-std`. Inside the future, we convert `asyncio` sleep into a Rust future and await it.


```toml
# Cargo.toml dependencies
[dependencies]
pyo3 = { version = "0.14" }
pyo3-asyncio = { version = "0.14", features = ["attributes", "async-std-runtime"] }
async-std = "1.9"
```

```rust
//! main.rs

use pyo3::prelude::*;

#[pyo3_asyncio::async_std::main]
async fn main() -> PyResult<()> {
    let fut = Python::with_gil(|py| {
        let asyncio = py.import("asyncio")?;
        // convert asyncio.sleep into a Rust Future
        pyo3_asyncio::async_std::into_future(asyncio.call_method1("sleep", (1.into_py(py),))?)
    })?;

    fut.await?;

    Ok(())
}
```

The same application can be written to use `tokio` instead using the `#[pyo3_asyncio::tokio::main]`
attribute.

```toml
# Cargo.toml dependencies
[dependencies]
pyo3 = { version = "0.14" }
pyo3-asyncio = { version = "0.14", features = ["attributes", "tokio-runtime"] }
tokio = "1.4"
```

```rust
//! main.rs

use pyo3::prelude::*;

#[pyo3_asyncio::tokio::main]
async fn main() -> PyResult<()> {
    let fut = Python::with_gil(|py| {
        let asyncio = py.import("asyncio")?;
        // convert asyncio.sleep into a Rust Future
        pyo3_asyncio::tokio::into_future(asyncio.call_method1("sleep", (1.into_py(py),))?)
    })?;

    fut.await?;

    Ok(())
}
```

More details on the usage of this library can be found in the [API docs](https://awestlake87.github.io/pyo3-asyncio/master/doc) and the primer below.

### PyO3 Native Rust Modules

PyO3 Asyncio can also be used to write native modules with async functions.

Add the `[lib]` section to `Cargo.toml` to make your library a `cdylib` that Python can import.
```toml
[lib]
name = "my_async_module"
crate-type = ["cdylib"]
```

Make your project depend on `pyo3` with the `extension-module` feature enabled and select your
`pyo3-asyncio` runtime:

For `async-std`:
```toml
[dependencies]
pyo3 = { version = "0.14", features = ["extension-module"] }
pyo3-asyncio = { version = "0.14", features = ["async-std-runtime"] }
async-std = "1.9"
```

For `tokio`:
```toml
[dependencies]
pyo3 = { version = "0.14", features = ["extension-module"] }
pyo3-asyncio = { version = "0.14", features = ["tokio-runtime"] }
tokio = "1.4"
```

Export an async function that makes use of `async-std`:

```rust
//! lib.rs

use pyo3::{prelude::*, wrap_pyfunction};

#[pyfunction]
fn rust_sleep(py: Python<'_>) -> PyResult<&Bound<'_, PyAny>>> {
    pyo3_asyncio::async_std::future_into_py(py, async {
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pymodule]
fn my_async_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rust_sleep, m)?)?;

    Ok(())
}
```

If you want to use `tokio` instead, here's what your module should look like:

```rust
//! lib.rs

use pyo3::{prelude::*, wrap_pyfunction};

#[pyfunction]
fn rust_sleep(py: Python<'_>) -> PyResult<&Bound<'_, PyAny>>> {
    pyo3_asyncio::tokio::future_into_py(py, async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pymodule]
fn my_async_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rust_sleep, m)?)?;
    Ok(())
}
```

You can build your module with maturin (see the [Using Rust in Python](https://pyo3.rs/main/#using-rust-from-python) section in the PyO3 guide for setup instructions). After that you should be able to run the Python REPL to try it out.

```bash
maturin develop && python3
🔗 Found pyo3 bindings
🐍 Found CPython 3.8 at python3
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
Python 3.8.5 (default, Jan 27 2021, 15:41:15)
[GCC 9.3.0] on linux
Type "help", "copyright", "credits" or "license" for more information.
>>> import asyncio
>>>
>>> from my_async_module import rust_sleep
>>>
>>> async def main():
>>>     await rust_sleep()
>>>
>>> # should sleep for 1s
>>> asyncio.run(main())
>>>
```

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
[`pyo3_asyncio::async_std::into_future`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/async_std/fn.into_future.html)
performs this conversion for us.

The following example uses `into_future` to call the `py_sleep` function shown above and then await the
coroutine object returned from the call:

```rust
use pyo3::prelude::*;

#[pyo3_asyncio::tokio::main]
async fn main() -> PyResult<()> {
    let future = Python::with_gil(|py| -> PyResult<_> {
        // import the module containing the py_sleep function
        let example = py.import("example")?;

        // calling the py_sleep method like a normal function
        // returns a coroutine
        let coroutine = example.call_method0("py_sleep")?;

        // convert the coroutine into a Rust future using the
        // tokio runtime
        pyo3_asyncio::tokio::into_future(coroutine)
    })?;

    // await the future
    future.await?;

    Ok(())
}
```

Alternatively, the below example shows how to write a `#[pyfunction]` which uses `into_future` to receive and await
a coroutine argument:

```rust
#[pyfunction]
fn await_coro(coro: &Bound<'_, PyAny>>) -> PyResult<()> {
    // convert the coroutine into a Rust future using the
    // async_std runtime
    let f = pyo3_asyncio::async_std::into_future(coro)?;

    pyo3_asyncio::async_std::run_until_complete(coro.py(), async move {
        // await the future
        f.await?;
        Ok(())
    })
}
```

This could be called from Python as:

```python
import asyncio

async def py_sleep():
    asyncio.sleep(1)

await_coro(py_sleep())
```

If for you wanted to pass a callable function to the `#[pyfunction]` instead, (i.e. the last line becomes `await_coro(py_sleep))`, then the above example needs to be tweaked to first call the callable to get the coroutine:

```rust
#[pyfunction]
fn await_coro(callable: &Bound<'_, PyAny>>) -> PyResult<()> {
    // get the coroutine by calling the callable
    let coro = callable.call0()?;

    // convert the coroutine into a Rust future using the
    // async_std runtime
    let f = pyo3_asyncio::async_std::into_future(coro)?;

    pyo3_asyncio::async_std::run_until_complete(coro.py(), async move {
        // await the future
        f.await?;
        Ok(())
    })
}
```

This can be particularly helpful where you need to repeatedly create and await a coroutine. Trying to await the same coroutine multiple times will raise an error:

```python
RuntimeError: cannot reuse already awaited coroutine
```

> If you're interested in learning more about `coroutines` and `awaitables` in general, check out the
> [Python 3 `asyncio` docs](https://docs.python.org/3/library/asyncio-task.html) for more information.

## Awaiting a Rust Future in Python

Here we have the same async function as before written in Rust using the
[`async-std`](https://async.rs/) runtime:

```rust
/// Sleep for 1 second
async fn rust_sleep() {
    async_std::task::sleep(std::time::Duration::from_secs(1)).await;
}
```

Similar to Python, Rust's async functions also return a special object called a
`Future`:

```rust
let future = rust_sleep();
```

We can convert this `Future` object into Python to make it `awaitable`. This tells Python that you
can use the `await` keyword with it. In order to do this, we'll call
[`pyo3_asyncio::async_std::future_into_py`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/async_std/fn.future_into_py.html):

```rust
use pyo3::prelude::*;

async fn rust_sleep() {
    async_std::task::sleep(std::time::Duration::from_secs(1)).await;
}

#[pyfunction]
fn call_rust_sleep(py: Python<'_>) -> PyResult<&Bound<'_, PyAny>>> {
    pyo3_asyncio::async_std::future_into_py(py, async move {
        rust_sleep().await;
        Ok(Python::with_gil(|py| py.None()))
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
thread must block on [`pyo3_asyncio::async_std::run_until_complete`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/async_std/fn.run_until_complete.html).

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

    let fut = Python::with_gil(|py| -> PyResult<_> {
        let asyncio = py.import("asyncio")?;

        // convert asyncio.sleep into a Rust Future
        pyo3_asyncio::async_std::into_future(
            asyncio.call_method1("sleep", (1.into_py(py),))?
        )
    })?;

    fut.await?;

    Ok(())
}
```

### A Note About `asyncio.run`

In Python 3.7+, the recommended way to run a top-level coroutine with `asyncio`
is with `asyncio.run`. In `v0.13` we recommended against using this function due to initialization issues, but in `v0.14` it's perfectly valid to use this function... with a caveat.

Since our Rust <--> Python conversions require a reference to the Python event loop, this poses a problem. Imagine we have a PyO3 Asyncio module that defines
a `rust_sleep` function like in previous examples. You might rightfully assume that you can call pass this directly into `asyncio.run` like this:

```python
import asyncio

from my_async_module import rust_sleep

asyncio.run(rust_sleep())
```

You might be surprised to find out that this throws an error:
```bash
Traceback (most recent call last):
  File "example.py", line 5, in <module>
    asyncio.run(rust_sleep())
RuntimeError: no running event loop
```

What's happening here is that we are calling `rust_sleep` _before_ the future is
actually running on the event loop created by `asyncio.run`. This is counter-intuitive, but expected behaviour, and unfortunately there doesn't seem to be a good way of solving this problem within PyO3 Asyncio itself.

However, we can make this example work with a simple workaround:

```python
import asyncio

from my_async_module import rust_sleep

# Calling main will just construct the coroutine that later calls rust_sleep.
# - This ensures that rust_sleep will be called when the event loop is running,
#   not before.
async def main():
    await rust_sleep()

# Run the main() coroutine at the top-level instead
asyncio.run(main())
```

### Non-standard Python Event Loops

Python allows you to use alternatives to the default `asyncio` event loop. One
popular alternative is `uvloop`. In `v0.13` using non-standard event loops was
a bit of an ordeal, but in `v0.14` it's trivial.

#### Using `uvloop` in a PyO3 Asyncio Native Extensions

```toml
# Cargo.toml

[lib]
name = "my_async_module"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.14", features = ["extension-module"] }
pyo3-asyncio = { version = "0.14", features = ["tokio-runtime"] }
async-std = "1.9"
tokio = "1.4"
```

```rust
//! lib.rs

use pyo3::{prelude::*, wrap_pyfunction};

#[pyfunction]
fn rust_sleep(py: Python<'_>) -> PyResult<&Bound<'_, PyAny>>> {
    pyo3_asyncio::tokio::future_into_py(py, async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pymodule]
fn my_async_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rust_sleep, m)?)?;

    Ok(())
}
```

```bash
$ maturin develop && python3
🔗 Found pyo3 bindings
🐍 Found CPython 3.8 at python3
    Finished dev [unoptimized + debuginfo] target(s) in 0.04s
Python 3.8.8 (default, Apr 13 2021, 19:58:26)
[GCC 7.3.0] :: Anaconda, Inc. on linux
Type "help", "copyright", "credits" or "license" for more information.
>>> import asyncio
>>> import uvloop
>>>
>>> import my_async_module
>>>
>>> uvloop.install()
>>>
>>> async def main():
...     await my_async_module.rust_sleep()
...
>>> asyncio.run(main())
>>>
```

#### Using `uvloop` in Rust Applications

Using `uvloop` in Rust applications is a bit trickier, but it's still possible
with relatively few modifications.

Unfortunately, we can't make use of the `#[pyo3_asyncio::<runtime>::main]` attribute with non-standard event loops. This is because the `#[pyo3_asyncio::<runtime>::main]` proc macro has to interact with the Python
event loop before we can install the `uvloop` policy.

```toml
[dependencies]
async-std = "1.9"
pyo3 = "0.14"
pyo3-asyncio = { version = "0.14", features = ["async-std-runtime"] }
```

```rust
//! main.rs

use pyo3::{prelude::*, types::PyType};

fn main() -> PyResult<()> {
    pyo3::prepare_freethreaded_python();

    Python::with_gil(|py| {
        let uvloop = py.import("uvloop")?;
        uvloop.call_method0("install")?;

        // store a reference for the assertion
        let uvloop = PyObject::from(uvloop);

        pyo3_asyncio::async_std::run(py, async move {
            // verify that we are on a uvloop.Loop
            Python::with_gil(|py| -> PyResult<()> {
                assert!(pyo3_asyncio::async_std::get_current_loop(py)?.is_instance(
                    uvloop
                        .as_ref(py)
                        .getattr("Loop")?
                )?);
                Ok(())
            })?;

            async_std::task::sleep(std::time::Duration::from_secs(1)).await;

            Ok(())
        })
    })
}
```

## Additional Information
- Managing event loop references can be tricky with pyo3-asyncio. See [Event Loop References](https://docs.rs/pyo3-asyncio/#event-loop-references) in the API docs to get a better intuition for how event loop references are managed in this library.
- Testing pyo3-asyncio libraries and applications requires a custom test harness since Python requires control over the main thread. You can find a testing guide in the [API docs for the `testing` module](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/testing)
