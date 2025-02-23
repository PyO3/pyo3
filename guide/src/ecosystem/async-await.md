# Using `async` and `await`

*`async`/`await` support is currently being integrated in PyO3. See the [dedicated documentation](../async-await.md)*

If you are working with a Python library that makes use of async functions or wish to provide
Python bindings for an async Rust library, [`pyo3-async-runtimes`](https://github.com/PyO3/pyo3-async-runtimes)
likely has the tools you need. It provides conversions between async functions in both Python and
Rust and was designed with first-class support for popular Rust runtimes such as
[`tokio`](https://tokio.rs/) and [`async-std`](https://async.rs/). In addition, all async Python
code runs on the default `asyncio` event loop, so `pyo3-async-runtimes` should work just fine with existing
Python libraries.

## Additional Information
- Managing event loop references can be tricky with `pyo3-async-runtimes`. See [Event Loop References](https://docs.rs/pyo3-async-runtimes/#event-loop-references-and-contextvars) in the API docs to get a better intuition for how event loop references are managed in this library.
- Testing `pyo3-async-runtimes` libraries and applications requires a custom test harness since Python requires control over the main thread. You can find a testing guide in the [API docs for the `testing` module](https://docs.rs/pyo3-async-runtimes/latest/pyo3_async_runtimes/testing)
