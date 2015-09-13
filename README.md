rust-cpython [![Build Status](https://travis-ci.org/dgrunwald/rust-cpython.svg?branch=master)](https://travis-ci.org/dgrunwald/rust-cpython)
====================

[Rust](http://www.rust-lang.org/) bindings for the [python](https://www.python.org/) interpreter.

* [Documentation](http://dgrunwald.github.io/rust-cpython/doc/cpython/)
* Cargo package: [cpython](https://crates.io/crates/cpython)

---

Copyright (c) 2015 Daniel Grunwald.
Rust-cpython is licensed under the [MIT license](http://opensource.org/licenses/MIT).
Python is licensed under the [Python License](https://docs.python.org/2/license.html).

Supported Python versions:
* Python 2.7
* Python 3.3
* Python 3.4
* Python 3.5

Supported Rust version:
* Rust nightly only :(

# Usage

To use `cpython`, add this to your `Cargo.toml`:

```toml
[dependencies]
cpython = { git = "https://github.com/dgrunwald/rust-cpython.git" }
```

Example program displaying the value of `sys.version`:

```rust
extern crate cpython;

use cpython::Python;
use cpython::ObjectProtocol; //for call method

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();

    let sys = py.import("sys").unwrap();
    let version: String = sys.get(py, "version").unwrap().extract(py).unwrap();

    let os = py.import("os").unwrap();
    let getenv = os.get(py, "getenv").unwrap();
    let user: String = getenv.call(py, ("USER",), None).unwrap().extract(py).unwrap();

    println!("Hello {}, I'm Python {}", user, version);
}
```

