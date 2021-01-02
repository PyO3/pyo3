# Advanced topics

## FFI

PyO3 exposes much of Python's C API through the `ffi` module.

The C API is naturally unsafe and requires you to manage reference counts, errors and specific invariants yourself. Please refer to the [C API Reference Manual](https://docs.python.org/3/c-api/) and [The Rustonomicon](https://doc.rust-lang.org/nightly/nomicon/ffi.html) before using any function from that API.

## Memory Management

PyO3's "owned references" (`&PyAny` etc.) make PyO3 more ergonomic to use by ensuring that their lifetime can never be longer than the duration the Python GIL is held. This means that most of PyO3's API can assume the GIL is held. (If PyO3 could not assume this, every PyO3 API would need to take a `Python` GIL token to prove that the GIL is held.)

The caveat to these "owned references" is that Rust references do not normally convey ownership (they are always `Copy`, and cannot implement `Drop`). Whenever a PyO3 API returns an owned reference, PyO3 stores it internally, so that PyO3 can decrease the reference count just before PyO3 releases the GIL.

For most use cases this behaviour is invisible. Occasionally, however, users may need to clear memory usage sooner than PyO3 usually does. PyO3 exposes this functionality with the  the `GILPool` struct. When a `GILPool` is dropped, ***all*** owned references created after the `GILPool` was created will be cleared.

The unsafe function `Python::new_pool` allows you to create a new `GILPool`. When doing this, you must be very careful to ensure that once the `GILPool` is dropped you do not retain access any owned references created after the `GILPool` was created.
