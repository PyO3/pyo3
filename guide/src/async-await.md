# Async / Await

Both Python and Rust have support for async functions, but bindings for these functions are not as 
straightforward as they are for blocking functions. Both languages have a fairly distinct model for 
async functions and Python's needs in particular can sometimes be restrictive.

[pyo3-asyncio](https://github.com/awestlake87/pyo3-asyncio) was created to provide conversions 
between async Python and async Rust as well as manage the nitty gritty details of Python's event 
loop.

## Awaiting an Async Python Function in Rust

Let's take a look at a dead simple async Python function:

```python
async def py_sleep():
    await asyncio.sleep(1)
```

This function simply sleeps for 1 second and returns. So what does this look like to PyO3? 

First, it helps to have a little background on Python's async functions. 
Async functions in python are simply functions that return a `coroutine` object. You can read more about 
`coroutine` objects in the [Python 3 docs](https://docs.python.org/3/library/asyncio-task.html), but
for our purposes, we really don't need to know much about them. The key factor here is that calling
an `async` function is _just like calling a regular function_, the only difference is that we have
to do something special with the object that it returns.

Normally in Python, that something special is the `await` keyword, but in Rust, we don't have the 
luxury of using Python's syntax. Luckily, Rust also has an `await` keyword that does something
similar, we just need to find a way of converting a `&PyAny` into a Rust future so we can use the
`await` keyword on it.

That's where pyo3-asyncio comes in. `pyo3_asyncio::into_future` performs this conversion for us:

```rust
let future = Python::with_gil(|py| {
    // import the module containing the py_sleep function
    let example = py.import("example")?;

    // calling the py_sleep method like a normal function returns a coroutine
    let coroutine = example.call_method0("py_sleep")?;

    // convert the coroutine into a Rust future
    pyo3_asyncio::into_future(py, coroutine)
})?;

// await the future
future.await;
```

## Awaiting a Rust Future in Python

Here we have the same async function as before written in Rust using the 
[`async-std`](https://async.rs/) runtime:

```rust
async fn rust_sleep() {
    async_std::task::sleep(Duration::from_secs(1)).await;
}
```

Similar to Python, Rust's async functions also return a special object called a
`Future`:

```rust
let future = rust_sleep();
```

We can convert this future object into Python to make it "Awaitable". This tells Python that you can
use the `await` keyword with it. In order to do this, we'll call 
`pyo3_asyncio::async_std::into_coroutine`:

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
Python's Asyncio features, like proper signal handling, require control over the main thread, which
doesn't always play well with Rust.

Luckily, Rust's event loops are pretty flexible and don't _need_ control over the main thread, so in
pyo3-asyncio, we decided the best way to handle Rust/Python interop was to just surrender the main
thread to Python and run Rust's event loops in the background. Unfortunately, since most event loop 
implementations _prefer_ control over the main thread, this can still make some things awkward.

### PyO3 Asyncio Initialization

Because Python needs to control the main thread, we can't use the convenient proc macros from Rust
runtimes to handle the main function or test functions. 

Instead, the initialization for PyO3 has to be done manually from the main function and the main 
thread must block on `pyo3_asyncio::run_forever` or `pyo3_asyncio::generic::run_until_complete`.
Because we have to block on one of those functions, we can't use `tokio::main` since it's not a good
idea to make long blocking calls during an async function.

In addition, some runtimes, such as Tokio, may require some additional initialization since their 
runtimes are customizable. For tokio, this initialization happens during the `tokio::main` proc 
macro, but since we can't use that for our purposes, it has to be initialized manually. See the 
`pyo3-asyncio` API docs for more information.

Here's a full example of PyO3 initialization:
```rust
use pyo3::prelude::*;

fn main() {
    // if using tokio, you should perform some additional initialization here:
    // pyo3_asyncio::tokio::init_multi_thread();

    Python::with_gil(|py| {
        // Initialize the runtime
        pyo3_asyncio::with_runtime(py, || {
            // Run the Python event loop until the given future completes
            pyo3_asyncio::async_std::run_until_complete(py, async {
                // PyO3 is initialized - Ready to go
                Ok(())
            })?;

            Ok(())
        })
        .map_err(|e| {
            e.print_and_set_sys_last_vars(py);  
        })
        .unwrap();
    })
}
```

## PyO3 Asyncio in Cargo Tests

The default Cargo Test harness does not currently allow test crates to provide their own main 
function, so there doesn't seem to be a good way to allow Python to gain control over the main
thread.

We can, however, override the default test harness and provide our own. `pyo3-asyncio` provides some
utilities to help us do just that!

### Creating A PyO3 Asyncio Integration Test

#### Main Test File
First, we need to create the test's main file. Although these tests are considered integration
tests, we cannot put them in the `tests` directory since that is a special directory owned by
Cargo. Instead, we put our tests in a `pytests` directory, although the name `pytests` is just
a convention.

`pytests/test_example.rs`
```rust
fn main() {

}
```

#### Test Manifest Entry
Next, we need to add our test file to the Cargo manifest. Add the following section to your
`Cargo.toml`

```toml
[[test]]
name = "test_example"
path = "pytests/test_example.rs"
harness = false
```

At this point you should be able to run the test via `cargo test`

#### Using the PyO3 Asyncio Test Harness
Now that we've got our test registered with `cargo test`, we can start using the PyO3 Asyncio
test harness.

In your `Cargo.toml` add the testing feature to `pyo3-asyncio` and select your preferred runtime:
```toml
pyo3-asyncio = { version = "0.13", features = ["testing", "async-std-runtime"] }
```

Now, in your test's main file, call [`crate::async_std::testing::test_main`]:

```rust
fn main() {
    pyo3_asyncio::async_std::testing::test_main("Example Test Suite", vec![]);
}
```

##### Tokio's Main Function

As we mentioned earlier, Tokio requires some additional initialization. If you're going to use the 
Tokio runtime, you'll need to call one of the initialization functions in the `pyo3_asyncio::tokio` 
module before running the Tokio `test_main`.

```rust
fn main() {
    pyo3_asyncio::tokio::init_multi_thread();
    pyo3_asyncio::tokio::testing::test_main("Example Test Suite", vec![]);
}
```


#### Adding Tests to the PyO3 Asyncio Test Harness

```rust
use std::{time::Duration, thread};

use pyo3_asyncio::testing::Test;

fn main() {
    pyo3_asyncio::async_std::testing::test_main(
        "Example Test Suite",
        vec![
            Test::new_async(
                "test_async_sleep".into(),
                async move {
                    async_std::task::sleep(Duration::from_secs(1)).await;
                    Ok(())
                }
            ),
            pyo3_asyncio::async_std::testing::new_sync_test(
                "test_sync_sleep".into(),
                || {
                    thread::sleep(Duration::from_secs(1));
                    Ok(())
                }
            )
        ]
    );
}
```