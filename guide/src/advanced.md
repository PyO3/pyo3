# Advanced topics

## ffi

PyO3 exposes much of Python's C API through the `ffi` module.

The C API is naturally unsafe and requires you to manage reference counts, errors and specific invariants yourself. Please refer to the [C API Reference Manual](https://docs.python.org/3/c-api/) and [The Rustonomicon](https://doc.rust-lang.org/nightly/nomicon/ffi.html) before using any function from that API.

## Testing

Currently, [#341](https://github.com/PyO3/pyo3/issues/341) causes `cargo test` to fail with weird linking errors when the `extension-module` feature is activated. For now you can work around this by making the `extension-module` feature optional and running the tests with `cargo test --no-default-features`:

```toml
[dependencies.pyo3]
version = "0.7.0"

[features]
extension-module = ["pyo3/extension-module"]
default = ["extension-module"]
```
