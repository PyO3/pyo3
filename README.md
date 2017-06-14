# PyO3 [![Build Status](https://travis-ci.org/PyO3/PyO3.svg?branch=master)](https://travis-ci.org/PyO3/PyO3)

[Rust](http://www.rust-lang.org/) bindings for the [python](https://www.python.org/) interpreter.

* [Documentation](http://pyo3.github.io/PyO3/doc/PyO3/)
* Cargo package: [pyo3](https://crates.io/crates/pyo3)

---

PyO3 is licensed under the [APACHE-2.0 license](http://opensource.org/licenses/APACHE-2.0).
Python is licensed under the [Python License](https://docs.python.org/2/license.html).

Supported Python versions:

* Python2.7, Python 3.5 and up

Supported Rust version:

* Rust 1.17.0-nightly or later
* On Windows, we require rustc 1.17.0-nightly

## Usage

To use `pyo3`, add this to your `Cargo.toml`:

```toml
[dependencies]
pyo3 = "0.1"
```

Example program displaying the value of `sys.version`:

```rust
extern crate pyo3;

use pyo3::{Python, PyDict, PyResult};

fn main() {
    let gil = Python::acquire_gil();
    hello(gil.python()).unwrap();
}

fn hello(py: Python) -> PyResult<()> {
    let sys = py.import("sys")?;
    let version: String = sys.get("version")?.extract(py)?;

    let locals = PyDict::new(py);
    locals.set_item("os", py.import("os")?)?;
    let user: String = py.eval("os.getenv('USER') or os.getenv('USERNAME')", None, Some(&locals))?.extract(py)?;

    println!("Hello {}, I'm Python {}", user, version);
    Ok(())
}
```

Example library with python bindings:

The following two files will build with `cargo build`, and will generate a python-compatible library.
On Mac OS, you will need to rename the output from \*.dylib to \*.so.
On Windows, you will need to rename the output from \*.dll to \*.pyd.

**`Cargo.toml`:**

```toml
[lib]
name = "rust2py"
crate-type = ["cdylib"]

[dependencies.pyo3]
version = "0.1"
features = ["extension-module"]
```

**`src/lib.rs`**

```rust
#![feature(proc_macro)]
#[macro_use] extern crate pyo3;

use pyo3::{py, PyResult, Python, PyModule};

// add bindings to the generated python module
// N.B: names: "librust2py" must be the name of the `.so` or `.pyd` file
#[py::modinit(rust2py)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    m.add(py, "__doc__", "This module is implemented in Rust.")?;

    #[pyfn(m, "sum_as_string")]
    // pyo3 aware function. All of our python interface could be
    // declared in a separate module.
    // Note that the py_fn!() macro automatically converts the arguments from
    // Python objects to Rust values; and the Rust return value back into a Python object.
    fn sum_as_string_py(_: Python, a:i64, b:i64) -> PyResult<String> {
       let out = sum_as_string(a, b);
       Ok(out)
    }

    Ok(())
}

// logic implemented as a normal rust function
fn sum_as_string(a:i64, b:i64) -> String {
    format!("{}", a + b).to_string()
}

```

For `setup.py` integration, see [setuptools-rust](https://github.com/PyO3/setuptools-rust)


**This is fork of rust-cpython project https://github.com/dgrunwald/rust-cpython**
