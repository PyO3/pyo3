# PyO3

[![Build Status](https://travis-ci.org/PyO3/pyo3.svg?branch=master)](https://travis-ci.org/PyO3/pyo3) [![Build Status](https://ci.appveyor.com/api/projects/status/github/PyO3/pyo3?branch=master&svg=true)](https://ci.appveyor.com/project/fafhrd91/pyo3) [![codecov](https://codecov.io/gh/PyO3/pyo3/branch/master/graph/badge.svg)](https://codecov.io/gh/PyO3/pyo3) [![crates.io](http://meritbadge.herokuapp.com/pyo3)](https://crates.io/crates/pyo3) [![Join the dev chat](https://img.shields.io/gitter/room/nwjs/nw.js.svg)](https://gitter.im/PyO3/Lobby)

[Rust](http://www.rust-lang.org/) bindings for the [Python](https://www.python.org/) interpreter. This includes running and interacting with python code from a rust binaries as well as writing native python modules.

* User Guide: [stable](https://pyo3.rs) | [master](https://pyo3.rs/master)
* [API Documentation](https://docs.rs/crate/pyo3/)

A comparison with rust-cpython can be found [in the guide](https://pyo3.rs/master/rust-cpython.html).

## Usage

Pyo3 supports python 2.7 as well as python 3.5 and up. The minimum required rust version is 1.27.0-nightly 2018-05-01.

### From a rust binary

To use `pyo3`, add this to your `Cargo.toml`:

```toml
[dependencies]
pyo3 = "0.3"
```

Example program displaying the value of `sys.version`:

```rust
#![feature(use_extern_macros, specialization)]

extern crate pyo3;

use pyo3::prelude::*;

fn main() -> PyResult<()> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let sys = py.import("sys")?;
    let version: String = sys.get("version")?.extract()?;

    let locals = PyDict::new(py);
    locals.set_item("os", py.import("os")?)?;
    let user: String = py.eval("os.getenv('USER') or os.getenv('USERNAME')", None, Some(&locals))?.extract()?;

    println!("Hello {}, I'm Python {}", user, version);
    Ok(())
}
```

### As native module

Pyo3 can be used to generate a python-compatible library.

**`Cargo.toml`:**

```toml
[package]
name = "rust2py"
version = "0.1.0"

[lib]
name = "rust2py"
crate-type = ["cdylib"]

[dependencies.pyo3]
version = "0.3"
features = ["extension-module"]
```

**`src/lib.rs`**

```rust
#![feature(use_extern_macros, specialization)]

extern crate pyo3;
use pyo3::prelude::*;



// Add bindings to the generated python module
// N.B: names: "librust2py" must be the name of the `.so` or `.pyd` file
/// This module is implemented in Rust.
#[pymodinit]
fn rust2py(py: Python, m: &PyModule) -> PyResult<()> {

    #[pyfn(m, "sum_as_string")]
    // ``#[pyfn()]` converts the arguments from Python objects to Rust values
    // and the Rust return value back into a Python object.
    fn sum_as_string_py(a:i64, b:i64) -> PyResult<String> {
       let out = sum_as_string(a, b);
       Ok(out)
    }

    Ok(())
}

// The logic can be implemented as a normal rust function
fn sum_as_string(a:i64, b:i64) -> String {
    format!("{}", a + b).to_string()
}
```

On windows and linux, you can build normally with `cargo build --release`. On Mac Os, you need to set additional linker arguments. One option is to compile with `cargo rustc --release -- -C link-arg=-undefined -C link-arg=dynamic_lookup`, the other is to create a `.cargo/config` with the following content: 

```toml
[target.x86_64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]
```

Also on macOS, you will need to rename the output from \*.dylib to \*.so. On Windows, you will need to rename the output from \*.dll to \*.pyd.

[`setuptools-rust`](https://github.com/PyO3/setuptools-rust) can be used to generate a python package and includes the commands above by default. See [examples/word-count](examples/word-count) and the associated setup.py.

## License

PyO3 is licensed under the [Apache-2.0 license](http://opensource.org/licenses/APACHE-2.0).
Python is licensed under the [Python License](https://docs.python.org/2/license.html).
