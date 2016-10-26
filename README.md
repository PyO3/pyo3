rust-cpython [![Build Status](https://travis-ci.org/dgrunwald/rust-cpython.svg?branch=master)](https://travis-ci.org/dgrunwald/rust-cpython)
====================

[Rust](http://www.rust-lang.org/) bindings for the [python](https://www.python.org/) interpreter.

* [Documentation](http://dgrunwald.github.io/rust-cpython/doc/cpython/)
* Cargo package: [cpython](https://crates.io/crates/cpython)

---

Copyright (c) 2015-2016 Daniel Grunwald.
Rust-cpython is licensed under the [MIT license](http://opensource.org/licenses/MIT).
Python is licensed under the [Python License](https://docs.python.org/2/license.html).

Supported Python versions:
* Python 2.7
* Python 3.3
* Python 3.4
* Python 3.5

Supported Rust version:
* Rust 1.7.0 or later

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

Example library with python bindings:

The following two files will build with `cargo build`, and will generate a python-compatible library. (On macOS, you will need to rename the output from \*.dynlib to \*.so)

**`Cargo.toml`:**
```toml
[lib]
name = "rust2py"
crate-type = ["dylib"]

[dependencies]
cpython = { git = "https://github.com/dgrunwald/rust-cpython.git" }
```

**`src/lib.rs`**
```rust
#[macro_use] extern crate cpython;

use cpython::{PyResult, Python};

// add bindings to the generated python module
// N.B: names: "rust2py" must be the lib name in Cargo.toml
py_module_initializer!(librust2py, initlibrust2py, PyInit_librust2py, |py, m| {
    try!(m.add(py, "__doc__", "This module is implemented in Rust."));
    try!(m.add(py, "sum_as_string", py_fn!(py, sum_as_string_py(a: i64, b:i64))));
    Ok(())
});

// logic implemented as a normal rust function
fn sum_as_string(a:i64, b:i64) -> String {
    format!("{}", a + b).to_string()
}

// rust-cpython aware function. All of our python interface could be
// declared in a separate module. 
fn sum_as_string_py(_: Python, a:i64, b:i64) -> PyResult<String> {
    let out = sum_as_string(a, b);
    Ok(out)
}
```
