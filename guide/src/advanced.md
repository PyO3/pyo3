# Advanced topics

## ffi

pyo3 exposes much of python's c api through the `ffi`.

The c api is naturually unsafe and requires you to manage reference counts, errors and specific invariants yourself. Please refer to the [C API Reference Manual](https://docs.python.org/3/c-api/) and [The Rustonomicon](https://doc.rust-lang.org/nightly/nomicon/ffi.html) before using any function from that api.