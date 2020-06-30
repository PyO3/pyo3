# Frequently Asked Questions / Troubleshooting

## I'm experiencing deadlocks using PyO3 with lazy_static or once_cell!

`lazy_static` and `once_cell::sync` both use locks to ensure that initialization is performed only by a single thread. Because the Python GIL is an additional lock this can lead to deadlocks in the following way:

1. A thread (thread A) which has acquired the Python GIL starts initialization of a `lazy_static` value.
2. The initialization code calls some Python API which temporarily releases the GIL e.g. `Python::import`.
3. Another thread (thread B) acquires the Python GIL and attempts to access the same `lazy_static` value.
4. Thread B is blocked, because it waits for `lazy_static`'s initialization to lock to release.
5. Thread A is blocked, because it waits to re-aquire the GIL which thread B still holds.
6. Deadlock.

PyO3 provides a struct [`GILOnceCell`] which works equivalently to `OnceCell` but relies solely on the Python GIL for thread safety. This means it can be used in place of `lazy_static` or `once_cell` where you are experiencing the deadlock described above. See the documentation for [`GILOnceCell`] for an example how to use it.

[`GILOnceCell`]: https://docs.rs/pyo3/latest/pyo3/once_cell/struct.GILOnceCell.html

## I can't run `cargo test`: I'm having linker issues like "Symbol not found" or "Undefined reference to _PyExc_SystemError"!

Currently, [#341](https://github.com/PyO3/pyo3/issues/341) causes `cargo test` to fail with linking errors when the `extension-module` feature is activated. For now you can work around this by making the `extension-module` feature optional and running the tests with `cargo test --no-default-features`:

```toml
[dependencies.pyo3]
version = "*"

[features]
extension-module = ["pyo3/extension-module"]
default = ["extension-module"]
```
