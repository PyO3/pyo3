# PyO3

[![Build Status](https://travis-ci.org/PyO3/pyo3.svg?branch=master)](https://travis-ci.org/PyO3/pyo3)
[![Actions Status](https://github.com/PyO3/pyo3/workflows/Test/badge.svg)](https://github.com/PyO3/pyo3/actions)
[![codecov](https://codecov.io/gh/PyO3/pyo3/branch/master/graph/badge.svg)](https://codecov.io/gh/PyO3/pyo3)
[![crates.io](http://meritbadge.herokuapp.com/pyo3)](https://crates.io/crates/pyo3)
[![minimum rustc 1.39](https://img.shields.io/badge/rustc-1.39+-blue.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![Join the dev chat](https://img.shields.io/gitter/room/nwjs/nw.js.svg)](https://gitter.im/PyO3/Lobby)

[Rust](http://www.rust-lang.org/) bindings for [Python](https://www.python.org/). This includes running and interacting with Python code from a Rust binary, as well as writing native Python modules.

* User Guide: [stable](https://pyo3.rs) | [master](https://pyo3.rs/master)

* API Documentation: [stable](https://docs.rs/pyo3/) |  [master](https://pyo3.rs/master/doc)

A comparison with rust-cpython can be found [in the guide](https://pyo3.rs/master/rust_cpython.html).

## Usage

PyO3 supports Python 3.5 and up. The minimum required Rust version is 1.39.0.

PyPy is also supported (via cpyext) for Python 3.5 only, targeted PyPy version is 7.0.0.
Please refer to the [pypy section in the guide](https://pyo3.rs/master/pypy.html).

You can either write a native Python module in Rust, or use Python from a Rust binary.

However, on some OSs, you need some additional packages. E.g. if you are on *Ubuntu 18.04*, please run

```bash
sudo apt install python3-dev python-dev
```

## Using Rust from Python

PyO3 can be used to generate a native Python module.

**`Cargo.toml`**

```toml
[package]
name = "string-sum"
version = "0.1.0"
edition = "2018"

[lib]
name = "string_sum"
crate-type = ["cdylib"]

[dependencies.pyo3]
version = "0.10.1"
features = ["extension-module"]
```

**`src/lib.rs`**

```rust
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust.
#[pymodule]
fn string_sum(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(sum_as_string))?;

    Ok(())
}
```

On Windows and Linux, you can build normally with `cargo build --release`. On macOS, you need to set additional linker arguments. One option is to compile with `cargo rustc --release -- -C link-arg=-undefined -C link-arg=dynamic_lookup`, the other is to create a `.cargo/config` with the following content:

```toml
[target.x86_64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]
```

While developing, you can symlink (or copy) and rename the shared library from the target folder: On MacOS, rename `libstring_sum.dylib` to `string_sum.so`, on Windows `libstring_sum.dll` to `string_sum.pyd`, and on Linux `libstring_sum.so` to `string_sum.so`. Then open a Python shell in the same folder and you'll be able to `import string_sum`.

Adding the `cdylib` arguments in the `Cargo.toml` files changes the way your crate is compiled.
Other Rust projects using your crate will have to link against the `.so` or `.pyd` file rather than include your library directly as normal.
In order to make available your crate in the usual way for Rust user, you you might want to consider using both `crate-type = ["cdylib", "rlib"]` so that Rust users can use the `rlib` (the default lib crate type).
Another possibility is to create a new crate to perform the binding.

To build, test and publish your crate as a Python module, you can use [maturin](https://github.com/PyO3/maturin) or [setuptools-rust](https://github.com/PyO3/setuptools-rust). You can find an example for setuptools-rust in [examples/word-count](https://github.com/PyO3/pyo3/tree/master/examples/word-count), while maturin should work on your crate without any configuration.

## Using Python from Rust

If you want your Rust application to create a Python interpreter internally and
use it to run Python code, add `pyo3` to your `Cargo.toml` like this:

```toml
[dependencies]
pyo3 = "0.10.1"
```

Example program displaying the value of `sys.version` and the current user name:

```rust
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

fn main() -> Result<(), ()> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    main_(py).map_err(|e| {
        // We can't display Python exceptions via std::fmt::Display,
        // so print the error here manually.
        e.print_and_set_sys_last_vars(py);
    })
}

fn main_(py: Python) -> PyResult<()> {
    let sys = py.import("sys")?;
    let version: String = sys.get("version")?.extract()?;
    let locals = [("os", py.import("os")?)].into_py_dict(py);
    let code = "os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";
    let user: String = py.eval(code, None, Some(&locals))?.extract()?;
    println!("Hello {}, I'm Python {}", user, version);
    Ok(())
}
```

Our guide has [a section](https://pyo3.rs/master/python_from_rust.html) with lots of examples
about this topic.

## Tools and libraries
 * [maturin](https://github.com/PyO3/maturin) _Zero configuration build tool for Rust-made Python extensions_.
 * [setuptools-rust](https://github.com/PyO3/setuptools-rust) _Setuptools plugin for Rust support_.
 * [pyo3-built](https://github.com/PyO3/pyo3-built) _Simple macro to expose metadata obtained with the [`built`](https://crates.io/crates/built) crate as a [`PyDict`](https://pyo3.github.io/pyo3/pyo3/struct.PyDict.html)_
 * [rust-numpy](https://github.com/PyO3/rust-numpy) _Rust binding of NumPy C-API_
 * [dict-derive](https://github.com/gperinazzo/dict-derive) _Derive FromPyObject to automatically transform Python dicts into Rust structs_

## Examples

 * [hyperjson](https://github.com/mre/hyperjson) _A hyper-fast Python module for reading/writing JSON data using Rust's serde-json_
 * [html-py-ever](https://github.com/PyO3/setuptools-rust/tree/master/html-py-ever) _Using [html5ever](https://github.com/servo/html5ever) through [kuchiki](https://github.com/kuchiki-rs/kuchiki) to speed up html parsing and css-selecting._
 * [point-process](https://github.com/ManifoldFR/point-process-rust/tree/master/pylib) _High level API for pointprocesses as a Python library_
 * [autopy](https://github.com/autopilot-rs/autopy) _A simple, cross-platform GUI automation library for Python and Rust._
   * Contains an example of building wheels on TravisCI and appveyor using [cibuildwheel](https://github.com/joerick/cibuildwheel)
 * [orjson](https://github.com/ijl/orjson)  _Fast Python JSON library_
 * [inline-python](https://github.com/dronesforwork/inline-python) _Inline Python code directly in your Rust code_
 * [Rogue-Gym](https://github.com/kngwyu/rogue-gym) _Customizable rogue-like game for AI experiments_
   * Contains an example of building wheels on Azure Pipelines
 * [fastuuid](https://github.com/thedrow/fastuuid/) _Python bindings to Rust's UUID library_
 * [python-ext-wasm](https://github.com/wasmerio/python-ext-wasm) _Python library to run WebAssembly binaries_
 * [mocpy](https://github.com/cds-astro/mocpy) _Astronomical Python library offering data structures for describing any arbitrary coverage regions on the unit sphere_
 * [tokenizers](https://github.com/huggingface/tokenizers/tree/master/bindings/python) _Python bindings to the Hugging Face tokenizers (NLP) written in Rust_

## License

PyO3 is licensed under the [Apache-2.0 license](http://opensource.org/licenses/APACHE-2.0).
Python is licensed under the [Python License](https://docs.python.org/2/license.html).
