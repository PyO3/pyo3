# Tracing

Python projects that write extension modules for performance reasons may want to
tap into [Rust's `tracing` ecosystem] to gain insight into the performance of
their extension module.

This section of the guide describes a few crates that provide ways to do that.
They build on [`tracing_subscriber`][tracing-subscriber] and require code
changes in both Python and Rust to integrate. Note that each extension module
must configure its own `tracing` integration; one extension module will not see
`tracing` data from a different module.

## `pyo3-tracing-subscriber` ([documentation][pyo3-tracing-subscriber-docs])

[`pyo3-tracing-subscriber`][pyo3-tracing-subscriber] provides a way for Python
projects to configure `tracing_subscriber`. It exposes a few
`tracing_subscriber` layers:
- `tracing_subscriber::fmt` for writing human-readable output to file or stdout
- `opentelemetry-stdout` for writing OTLP output to file or stdout
- `opentelemetry-otlp` for writing OTLP output to an OTLP endpoint

The extension module must call [`pyo3_tracing_subscriber::add_submodule`][add-submodule]
to export the Python classes needed to configure and initialize `tracing`.

On the Python side, use the `Tracing` context manager to initialize tracing and
run Rust code inside the context manager's block. `Tracing` takes a
`GlobalTracingConfig` instance describing the layers to be used.

See [the README on crates.io][pyo3-tracing-subscriber]
for example code.

## `pyo3-python-tracing-subscriber` ([documentation][pyo3-python-tracing-subscriber-docs])

The similarly-named [`pyo3-python-tracing-subscriber`][pyo3-python-tracing-subscriber]
implements a shim in Rust that forwards `tracing` data to a `Layer`
implementation defined in and passed in from Python.

There are many ways an extension module could integrate `pyo3-python-tracing-subscriber`
but a simple one may look something like this:
```rust,no_run
#[tracing::instrument]
#[pyfunction]
fn fibonacci(index: usize, use_memoized: bool) -> PyResult<usize> {
    // ...
}

#[pyfunction]
pub fn initialize_tracing(py_impl: Bound<'_, PyAny>) {
    tracing_subscriber::registry()
        .with(pyo3_python_tracing_subscriber::PythonCallbackLayerBridge::new(py_impl))
        .init();
}
```
The extension module must provide some way for Python to pass in one or more
Python objects that implement [the `Layer` interface]. Then it should construct
[`pyo3_python_tracing_subscriber::PythonCallbackLayerBridge`][PythonCallbackLayerBridge]
instances with each of those Python objects and initialize `tracing_subscriber`
as shown above.

The Python objects implement a modified version of the `Layer` interface:
- `on_new_span()` may return some state that will stored inside the Rust span
- other callbacks will be given that state as an additional positional argument

A dummy `Layer` implementation may look like this:
```python
import rust_extension

class MyPythonLayer:
    def __init__(self):
        pass

    # `on_new_span` can return some state
    def on_new_span(self, span_attrs: str, span_id: str) -> int:
        print(f"[on_new_span]: {span_attrs} | {span_id}")
        return random.randint(1, 1000)

    # The state from `on_new_span` is passed back into other trait methods
    def on_event(self, event: str, state: int):
        print(f"[on_event]: {event} | {state}")

    def on_close(self, span_id: str, state: int):
        print(f"[on_close]: {span_id} | {state}")

    def on_record(self, span_id: str, values: str, state: int):
        print(f"[on_record]: {span_id} | {values} | {state}")

def main():
    rust_extension.initialize_tracing(MyPythonLayer())

    print("10th fibonacci number: ", rust_extension.fibonacci(10, True))
```

`pyo3-python-tracing-subscriber` has [working examples]
showing both the Rust side and the Python side of an integration.

[pyo3-tracing-subscriber]: https://crates.io/crates/pyo3-tracing-subscriber
[pyo3-tracing-subscriber-docs]: https://docs.rs/pyo3-tracing-subscriber
[add-submodule]: https://docs.rs/pyo3-tracing-subscriber/*/pyo3_tracing_subscriber/fn.add_submodule.html

[pyo3-python-tracing-subscriber]: https://crates.io/crates/pyo3-python-tracing-subscriber
[pyo3-python-tracing-subscriber-docs]: https://docs.rs/pyo3-python-tracing-subscriber
[PythonCallbackLayerBridge]: https://docs.rs/pyo3-python-tracing-subscriber/*/pyo3_python_tracing_subscriber/struct.PythonCallbackLayerBridge.html
[working examples]: https://github.com/getsentry/pyo3-python-tracing-subscriber/tree/main/demo

[Rust's `tracing` ecosystem]: https://crates.io/crates/tracing
[tracing-subscriber]: https://docs.rs/tracing-subscriber/*/tracing_subscriber/
[the `Layer` interface]: https://docs.rs/tracing-subscriber/*/tracing_subscriber/layer/trait.Layer.html
