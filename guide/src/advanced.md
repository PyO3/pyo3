# Advanced topics

## FFI

PyO3 exposes much of Python's C API through the `ffi` module.

The C API is naturally unsafe and requires you to manage reference counts, errors and specific invariants yourself.
Please refer to the [C API Reference Manual](https://docs.python.org/3/c-api/) and [The Rustonomicon](https://doc.rust-lang.org/nightly/nomicon/ffi.html) before using any function from that API.

## Sharing types between multiple PyO3 modules

It is possible (but complicated) to share types between multiple PyO3 packages which can be compiled and installed separately.
This allows for ecosystems of functionality to be built similar to the many packages built on top of NumPy.
While PyO3 does not yet have any built-in support for doing this, the next sub-chapter of this guide describes the general approach to doing this, as well as the safety limitations to be aware of.

## Rust doctests

If using Python APIs during Rust doctests, be aware that since Rust 2024 introduced "merged doctests" it's possible for doctests which modify Python global state to affect other doctests' execution.
The [`standalone_crate`](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html#standalone_crate) marker can be used to run a doctest in its own process.

For example, [`append_to_inittab!`]({{#PYO3_DOCS_URL}}/pyo3/macro.append_to_inittab.html) must be called before Python is initialized.
Without `standalone_crate`, another doctest could initialize Python first, causing this example to panic:

```rust,ignore
/// ```rust,standalone_crate
/// use pyo3::prelude::*;
///
/// #[pymodule]
/// mod example {}
///
/// pyo3::append_to_inittab!(example);
/// Python::initialize();
/// Python::attach(|py| {
///     py.import("example").unwrap();
/// });
/// ```
```
