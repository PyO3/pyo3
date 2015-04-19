rust-cpython [![Build Status](https://travis-ci.org/dgrunwald/rust-cpython.svg?branch=master)](https://travis-ci.org/dgrunwald/rust-cpython)
====================

[Rust](http://www.rust-lang.org/) bindings for the [python](https://www.python.org/) interpreter.

* [Documentation](http://www.rust-ci.org/dgrunwald/rust-cpython/doc/cpython/)
* Cargo package: [cpython](https://crates.io/crates/cpython)

---

Copyright (c) 2015 Daniel Grunwald.
Rust-cpython is licensed under the [MIT license](http://opensource.org/licenses/MIT).
Python is licensed under the [Python License](https://docs.python.org/2/license.html).


# Usage

[`cpython` is available on crates.io](https://crates.io/crates/cpython) so you can use it like this (in your `Cargo.toml`):

```toml
[dependencies.cpython]
version = "*"
```

Example program displaying the value of `sys.version`:

```rust
extern crate cpython;

use cpython::{PythonObject, Python};

fn main() {
    let gil_guard = Python::acquire_gil();
    let py = gil_guard.python();
    let sys = py.import("sys").unwrap();
    let version = sys.get("version").unwrap().extract::<String>().unwrap();
    println!("Hello Python {}", version);
}
```

