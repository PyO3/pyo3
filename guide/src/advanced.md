# Advanced topics

## ffi

PyO3 exposes much of python's C api through the `ffi`.

The C api is naturally unsafe and requires you to manage reference counts, errors and specific invariants yourself. Please refer to the [C API Reference Manual](https://docs.python.org/3/c-api/) and [The Rustonomicon](https://doc.rust-lang.org/nightly/nomicon/ffi.html) before using any function from that api.