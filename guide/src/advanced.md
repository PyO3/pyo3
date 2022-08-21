# Advanced topics

## FFI

PyO3 exposes much of Python's C API through the `ffi` module.

The C API is naturally unsafe and requires you to manage reference counts, errors and specific invariants yourself. Please refer to the [C API Reference Manual](https://docs.python.org/3/c-api/) and [The Rustonomicon](https://doc.rust-lang.org/nightly/nomicon/ffi.html) before using any function from that API.

## Memory management

PyO3's `&PyAny` "owned references" and `Py<PyAny>` smart pointers are used to
access memory stored in Python's heap.  This memory sometimes lives for longer
than expected because of differences in Rust and Python's memory models.  See
the chapter on [memory management](./memory.md) for more information.
