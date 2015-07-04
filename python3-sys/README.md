rust-python3-sys
====================

[Rust](http://www.rust-lang.org/) FFI declarations for Python 3.
Supports the PEP 384 stable ABI for Python 3.3 or higher.

---

This [cargo -sys package](http://doc.crates.io/build-script.html#*-sys-packages) provides `python3` declarations.
Licensed under the Python license (see `LICENSE`).

For a safe high-level API, see [rust-cpython](https://github.com/dgrunwald/rust-cpython).

# Usage

[`python3-sys` is available on crates.io](https://crates.io/crates/python3-sys) so you can use it like this (in your `Cargo.toml`):

```toml
[dependencies.python3-sys]
version = "*"
```

In Rust, import the crate like this:

```rust
extern crate python3_sys as py;
```

Documentation for the python API is available on [https://docs.python.org/3/c-api/].

