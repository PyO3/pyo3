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

## I can't run `cargo test`: my crate cannot be found for tests in `tests/` directory!

The Rust book suggests to [put integration tests inside a `tests/` directory](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests).

For a PyO3 `extension-module` project where the `crate-type` is set to `"cdylib"` in your `Cargo.toml`,
the compiler won't be able to find your crate and will display errors such as `E0432` or `E0463`:

```
error[E0432]: unresolved import `my_crate`
 --> tests/test_my_crate.rs:1:5
  |
1 | use my_crate;
  |     ^^^^^^^^^^^^ no external crate `my_crate`
```

The best solution is to make your crate types include both `rlib` and `cdylib`:

```
# Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]
```

## Ctrl-C doesn't do anything while my Rust code is executing!

This is because Ctrl-C raises a SIGINT signal, which is handled by the calling Python process by simply setting a flag to action upon later. This flag isn't checked while Rust code called from Python is executing, only once control returns to the Python interpreter.

You can give the Python interpreter a chance to process the signal properly by calling `Python::check_signals`. It's good practice to call this function regularly if you have a long-running Rust function so that your users can cancel it.
