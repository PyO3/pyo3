rust-python27-sys [![Build Status](https://travis-ci.org/dgrunwald/rust-python27-sys.svg?branch=master)](https://travis-ci.org/dgrunwald/rust-python27-sys)
====================

[Rust](http://www.rust-lang.org/) FFI declarations for Python 2.7.

---

This [cargo -sys package](http://doc.crates.io/build-script.html#*-sys-packages) provides `python27` declarations.
Licensed under the Python license (see `LICENSE`).

For a safe high-level API, see [rust-cpython](https://github.com/dgrunwald/rust-cpython).

# Usage

[`python27-sys` is available on crates.io](https://crates.io/crates/python27-sys) so you can use it like this (in your `Cargo.toml`):

```toml
[dependencies.python27-sys]
version = "*"
```

In Rust, import the crate like this:

```rust
extern crate python27_sys as py;
```

Documentation for the python API is available on [https://docs.python.org/2/c-api/].

