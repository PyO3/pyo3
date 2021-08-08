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
pyo3 = { version = "0.13", features = ["extension-module"] }
pyo3-asyncio = { version = "0.14", features = ["async-std-runtime"] }
async-std = "1.9"
```

For `tokio`:
```toml
[dependencies]
pyo3 = { version = "0.13", features = ["extension-module"] }
pyo3-asyncio = { version = "0.14", features = ["tokio-runtime"] }
tokio = "1.4"
```

Export an async function that makes use of `async-std`:

```rust
//! lib.rs

use pyo3::{prelude::*, wrap_pyfunction};

#[pyfunction]
fn rust_sleep(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::async_std::future_into_py(py, async {
        async_std::task::sleep(std::time::Duration::from_secs(1)).await;
        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pymodule]
fn my_async_module(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rust_sleep, m)?)?;

    Ok(())
}

```

If you want to use `tokio` instead, here's what your module should look like:

```rust
//! lib.rs

use pyo3::{prelude::*, wrap_pyfunction};

#[pyfunction]
fn rust_sleep(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pymodule]
fn my_async_module(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rust_sleep, m)?)?;
    Ok(())
}

```

Build your module and rename `libmy_async_module.so` to `my_async_module.so`
```bash
cargo build --release && mv target/release/libmy_async_module.so target/release/my_async_module.so
```

Now, point your `PYTHONPATH` to the directory containing `my_async_module.so`, then you'll be able 
to import and use it:

```bash
$ PYTHONPATH=target/release python3
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
[`pyo3_asyncio::into_future`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/fn.into_future.html) 
performs this conversion for us:


```rust no_run
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

```rust compile_fail
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
fn call_rust_sleep(py: Python) -> PyResult<&PyAny> {
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
thread must block on [`pyo3_asyncio::run_forever`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/fn.run_forever.html) or [`pyo3_asyncio::async_std::run_until_complete`](https://docs.rs/pyo3-asyncio/latest/pyo3_asyncio/async_std/fn.run_until_complete.html).

Because we have to block on one of those functions, we can't use [`#[async_std::main]`](https://docs.rs/async-std/latest/async_std/attr.main.html) or [`#[tokio::main]`](https://docs.rs/tokio/1.1.0/tokio/attr.main.html)
since it's not a good idea to make long blocking calls during an async function.

> Internally, these `#[main]` proc macros are expanded to something like this:
> ```rust compile_fail
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
```rust no_run
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
  File "<stdin>", line 1, in <module>
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
pyo3 = { version = "0.14", features = ["extension-module", "auto-initialize"] }
pyo3-asyncio = { version = "0.14", features = ["tokio-runtime"] }
async-std = "1.9"
tokio = "1.4"
```

```rust
//! lib.rs

use pyo3::{prelude::*, wrap_pyfunction};

#[pyfunction]
fn rust_sleep(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pymodule]
fn my_async_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(rust_sleep, m)?)?;

    Ok(())
}
```

```bash
$ cargo build --release && mv target/release/libmy_async_module.so my_async_module.so
   Compiling pyo3-asyncio-lib v0.1.0 (pyo3-asyncio-lib)
    Finished release [optimized] target(s) in 1.00s
$ PYTHONPATH=target/release/ python3
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

> Unfortunately, we can't make use of the `#[pyo3_asyncio::<runtime>::main]` attribute with non-standard event loops. This is because the `#[pyo3_asyncio::<runtime>::main]` proc macro has to interact with the Python
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
                assert!(uvloop
                    .as_ref(py)
                    .getattr("Loop")?
                    .downcast::<PyType>()
                    .unwrap()
                    .is_instance(pyo3_asyncio::async_std::get_current_loop(py)?)?);
                Ok(())
            })?;

            async_std::task::sleep(std::time::Duration::from_secs(1)).await;

            Ok(())
        })
    })
}
```

### Event Loop References and Thread-awareness

One problem that arises when interacting with Python's asyncio library is that the functions we use to get a reference to the Python event loop can only be called in certain contexts. Since PyO3 Asyncio needs to interact with Python's event loop during conversions, the context of these conversions can matter a lot. 

> The core conversions we've mentioned so far in this guide should insulate you from these concerns in most cases, but in the event that they don't, this section should provide you with the information you need to solve these problems.

#### The Main Dilemma

Python programs can have many independent event loop instances throughout the lifetime of the application (`asyncio.run` for example creates its own event loop each time it's called for instance), and they can even run concurrent with other event loops. For this reason, the most correct method of obtaining a reference to the Python event loop is via `asyncio.get_running_loop`.

`asyncio.get_running_loop` returns the event loop associated with the current OS thread. It can be used inside Python coroutines to spawn concurrent tasks, interact with timers, or in our case signal between Rust and Python. This is all well and good when we are operating on a Python thread, but since Rust threads are not associated with a Python event loop, `asyncio.get_running_loop` will fail when called on a Rust runtime.

#### The Solution

A really straightforward way of dealing with this problem is to pass a reference to the associated Python event loop for every conversion. That's why in `v0.14`, we introduced a new set of conversion functions that do just that:

- `pyo3_asyncio::into_future_with_loop` - Convert a Python awaitable into a Rust future with the given asyncio event loop.
- `pyo3_asyncio::<runtime>::future_into_py_with_loop` - Convert a Rust future into a Python awaitable with the given asyncio event loop.
- `pyo3_asyncio::<runtime>::local_future_into_py_with_loop` - Convert a `!Send` Rust future into a Python awaitable with the given asyncio event loop.

One clear disadvantage to this approach (aside from the verbose naming) is that the Rust application has to explicitly track its references to the Python event loop. In native libraries, we can't make any assumptions about the underlying event loop, so the only reliable way to make sure our conversions work properly is to store a reference to the current event loop at the callsite to use later on.

```rust
use pyo3::prelude::*;

#[pyfunction]
fn sleep(py: Python) -> PyResult<&PyAny> {
    let current_loop = pyo3_asyncio::get_running_loop(py)?;
    let loop_ref = PyObject::from(current_loop);

    // Convert the async move { } block to a Python awaitable
    pyo3_asyncio::tokio::future_into_py_with_loop(current_loop, async move {
        let py_sleep = Python::with_gil(|py| {
            // Sometimes we need to call other async Python functions within
            // this future. In order for this to work, we need to track the 
            // event loop from earlier.
            pyo3_asyncio::into_future_with_loop(
                loop_ref.as_ref(py), 
                py.import("asyncio")?.call_method1("sleep", (1,))?
            )
        })?;

        py_sleep.await?;

        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pymodule]
fn my_mod(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sleep, m)?)?;
    Ok(())
}
```

> A naive solution to this tracking problem would be to cache a global reference to the asyncio event loop that all PyO3 Asyncio conversions can use. In fact this is what we did in PyO3 Asyncio `v0.13`. This works well for applications, but it soon became clear that this is not so ideal for libraries. Libraries usually have no direct control over how the event loop is managed, they're just expected to work with any event loop at any point in the application. This problem is compounded further when multiple event loops are used in the application since the global reference will only point to one.

Another disadvantage to this explicit approach that is less obvious is that we can no longer call our `#[pyfunction] fn sleep` on a Rust runtime since `asyncio.get_running_loop` only works on Python threads! It's clear that we need a slightly more flexible approach.

In order to detect the Python event loop at the callsite, we need something like `asyncio.get_running_loop` that works for _both Python and Rust_. In Python, `asyncio.get_running_loop` uses thread-local data to retrieve the event loop associated with the current thread. What we need in Rust is something that can retrieve the Python event loop associated with the current _task_.

Enter `pyo3_asyncio::<runtime>::get_current_loop`. This function first checks task-local data for a Python event loop, then falls back on `asyncio.get_running_loop` if no task-local event loop is found. This way both bases are covered.

Now, all we need is a way to store the event loop in task-local data. Since this is a runtime-specific feature, you can find the following functions in each runtime module:

- `pyo3_asyncio::<runtime>::scope` - Store the event loop in task-local data when executing the given Future.
- `pyo3_asyncio::<runtime>::scope_local` - Store the event loop in task-local data when executing the given `!Send` Future.

With these new functions, we can make our previous example more correct:

```rust no_run
use pyo3::prelude::*;

#[pyfunction]
fn sleep(py: Python) -> PyResult<&PyAny> {
    // get the current event loop through task-local data 
    // OR `asyncio.get_running_loop`
    let current_loop = pyo3_asyncio::tokio::get_current_loop(py)?;

    pyo3_asyncio::tokio::future_into_py_with_loop(
        current_loop, 
        // Store the current loop in task-local data 
        pyo3_asyncio::tokio::scope(current_loop.into(), async move {
            let py_sleep = Python::with_gil(|py| {
                pyo3_asyncio::into_future_with_loop(
                    // Now we can get the current loop through task-local data
                    pyo3_asyncio::tokio::get_current_loop(py)?, 
                    py.import("asyncio")?.call_method1("sleep", (1,))?
                )
            })?;

            py_sleep.await?;

            Ok(Python::with_gil(|py| py.None()))
        })
    )
}

#[pyfunction]
fn wrap_sleep(py: Python) -> PyResult<&PyAny> {
    // get the current event loop through task-local data 
    // OR `asyncio.get_running_loop`
    let current_loop = pyo3_asyncio::tokio::get_current_loop(py)?;

    pyo3_asyncio::tokio::future_into_py_with_loop(
        current_loop, 
        // Store the current loop in task-local data 
        pyo3_asyncio::tokio::scope(current_loop.into(), async move {
            let py_sleep = Python::with_gil(|py| {
                pyo3_asyncio::into_future_with_loop(
                    pyo3_asyncio::tokio::get_current_loop(py)?, 
                    // We can also call sleep within a Rust task since the
                    // event loop is stored in task local data
                    sleep(py)?
                )
            })?;

            py_sleep.await?;

            Ok(Python::with_gil(|py| py.None()))
        })
    )
}

#[pymodule]
fn my_mod(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sleep, m)?)?;
    m.add_function(wrap_pyfunction!(wrap_sleep, m)?)?;
    Ok(())
}
```

Even though this is more correct, it's clearly not more ergonomic. That's why we introduced a new set of functions with this functionality baked in:

- `pyo3_asyncio::<runtime>::into_future` 
  > Convert a Python awaitable into a Rust future (using `pyo3_asyncio::<runtime>::get_current_loop`)
- `pyo3_asyncio::<runtime>::future_into_py` 
  > Convert a Rust future into a Python awaitable (using `pyo3_asyncio::<runtime>::get_current_loop` and `pyo3_asyncio::<runtime>::scope` to set the task-local event loop for the given Rust future)
- `pyo3_asyncio::<runtime>::local_future_into_py` 
  > Convert a `!Send` Rust future into a Python awaitable (using `pyo3_asyncio::<runtime>::get_current_loop` and `pyo3_asyncio::<runtime>::scope_local` to set the task-local event loop for the given Rust future).

__These are the functions that we recommend using__. With these functions, the previous example can be rewritten to be more compact:

```rust
use pyo3::prelude::*;

#[pyfunction]
fn sleep(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let py_sleep = Python::with_gil(|py| {
            pyo3_asyncio::tokio::into_future(
                py.import("asyncio")?.call_method1("sleep", (1,))?
            )
        })?;

        py_sleep.await?;

        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pyfunction]
fn wrap_sleep(py: Python) -> PyResult<&PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async move {
        let py_sleep = Python::with_gil(|py| {
            pyo3_asyncio::tokio::into_future(sleep(py)?)
        })?;

        py_sleep.await?;

        Ok(Python::with_gil(|py| py.None()))
    })
}

#[pymodule]
fn my_mod(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sleep, m)?)?;
    m.add_function(wrap_pyfunction!(wrap_sleep, m)?)?;
    Ok(())
}
```

### A Note for `v0.13` Users

Hey guys, I realize that these are pretty major changes for `v0.14`, and I apologize in advance for having to modify the public API so much. I hope
the explanation above gives some much needed context and justification for all the breaking changes.

Part of the reason why it's taken so long to push out a `v0.14` release is because I wanted to make sure we got this release right. There were a lot of issues with the `v0.13` release that I hadn't anticipated, and it's thanks to your feedback and patience that we've worked through these issues to get a more correct, more flexible version out there!

This new release should address most the core issues that users have reported in the `v0.13` release, so I think we can expect more stability going forward.

Also, a special thanks to [@ShadowJonathan](https://github.com/ShadowJonathan) for helping with the design and review
of these changes!

- [@awestlake87](https://github.com/awestlake87)

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