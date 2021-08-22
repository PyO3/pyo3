# PyO3

[![actions status](https://github.com/PyO3/pyo3/workflows/CI/badge.svg)](https://github.com/PyO3/pyo3/actions)
[![benchmark](https://github.com/PyO3/pyo3/actions/workflows/bench.yml/badge.svg)](https://pyo3.rs/dev/bench/)
[![codecov](https://codecov.io/gh/PyO3/pyo3/branch/main/graph/badge.svg)](https://codecov.io/gh/PyO3/pyo3)
[![crates.io](https://img.shields.io/crates/v/pyo3)](https://crates.io/crates/pyo3)
[![minimum rustc 1.41](https://img.shields.io/badge/rustc-1.41+-blue.svg)](https://rust-lang.github.io/rfcs/2495-min-rust-version.html)
[![dev chat](https://img.shields.io/gitter/room/nwjs/nw.js.svg)](https://gitter.im/PyO3/Lobby)
[![contributing notes](https://img.shields.io/badge/contribute-on%20github-Green)](https://github.com/PyO3/pyo3/blob/main/Contributing.md)

[Rust](http://www.rust-lang.org/) bindings for [Python](https://www.python.org/), including tools for creating native Python extension modules. Running and interacting with Python code from a Rust binary is also supported.

- User Guide: [stable](https://pyo3.rs) | [main](https://pyo3.rs/main)

- API Documentation: [stable](https://docs.rs/pyo3/) | [main](https://pyo3.rs/main/doc)

## Usage

PyO3 supports the following software versions:
  - Python 3.6 and up (CPython and PyPy)
  - Rust 1.41 and up

You can use PyO3 to write a native Python module in Rust, or to embed Python in a Rust binary. The following sections explain each of these in turn.

### Using Rust from Python

PyO3 can be used to generate a native Python module. The easiest way to try this out for the first time is to use [`maturin`](https://github.com/PyO3/maturin). `maturin` is a tool for building and publishing Rust-based Python packages with minimal configuration. The following steps set up some files for an example Python module, install `maturin`, and then show how build and import the Python module.

First, create a new folder (let's call it `string_sum`) containing the following two files:

**`Cargo.toml`**

```toml
[package]
name = "string-sum"
version = "0.1.0"
edition = "2018"

[lib]
name = "string_sum"
# "cdylib" is necessary to produce a shared library for Python to import from.
#
# Downstream Rust code (including code in `bin/`, `examples/`, and `tests/`) will not be able
# to `use string_sum;` unless the "rlib" or "lib" crate type is also included, e.g.:
# crate-type = ["cdylib", "rlib"]
crate-type = ["cdylib"]

[dependencies.pyo3]
version = "0.14.3"
features = ["extension-module"]
```

**`src/lib.rs`**

```rust
use pyo3::prelude::*;

/// Formats the sum of two numbers as string.
#[pyfunction]
fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
    Ok((a + b).to_string())
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn string_sum(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;

    Ok(())
}
```

With those two files in place, now `maturin` needs to be installed. This can be done using Python's package manager `pip`. First, load up a new Python `virtualenv`, and install `maturin` into it:

```bash
$ cd string_sum
$ python -m venv .env
$ source .env/bin/activate
$ pip install maturin
```

Now build and execute the module:

```bash
$ maturin develop
# lots of progress output as maturin runs the compilation...
$ python
>>> import string_sum
>>> string_sum.sum_as_string(5, 20)
'25'
```

As well as with `maturin`, it is possible to build using [`setuptools-rust`](https://github.com/PyO3/setuptools-rust) or [manually](https://pyo3.rs/latest/building_and_distribution.html#manual-builds). Both offer more flexibility than `maturin` but require further configuration.

### Using Python from Rust

To embed Python into a Rust binary, you need to ensure that your Python installation contains a shared library. The following steps demonstrate how to ensure this (for Ubuntu), and then give some example code which runs an embedded Python interpreter.

To install the Python shared library on Ubuntu:

```bash
sudo apt install python3-dev
```

Start a new project with `cargo new` and add  `pyo3` to the `Cargo.toml` like this:

```toml
[dependencies.pyo3]
version = "0.14.3"
features = ["auto-initialize"]
```

Example program displaying the value of `sys.version` and the current user name:

```rust
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let sys = py.import("sys")?;
        let version: String = sys.get("version")?.extract()?;

        let locals = [("os", py.import("os")?)].into_py_dict(py);
        let code = "os.getenv('USER') or os.getenv('USERNAME') or 'Unknown'";
        let user: String = py.eval(code, None, Some(&locals))?.extract()?;

        println!("Hello {}, I'm Python {}", user, version);
        Ok(())
    })
}
```

The guide has [a section](https://pyo3.rs/latest/python_from_rust.html) with lots of examples
about this topic.

## Tools and libraries

- [maturin](https://github.com/PyO3/maturin) _Zero configuration build tool for Rust-made Python extensions_.
- [setuptools-rust](https://github.com/PyO3/setuptools-rust) _Setuptools plugin for Rust support_.
- [pyo3-built](https://github.com/PyO3/pyo3-built) _Simple macro to expose metadata obtained with the [`built`](https://crates.io/crates/built) crate as a [`PyDict`](https://docs.rs/pyo3/0.12.0/pyo3/types/struct.PyDict.html)_
- [rust-numpy](https://github.com/PyO3/rust-numpy) _Rust binding of NumPy C-API_
- [dict-derive](https://github.com/gperinazzo/dict-derive) _Derive FromPyObject to automatically transform Python dicts into Rust structs_
- [pyo3-log](https://github.com/vorner/pyo3-log) _Bridge from Rust to Python logging_
- [pythonize](https://github.com/davidhewitt/pythonize) _Serde serializer for converting Rust objects to JSON-compatible Python objects_
- [pyo3-asyncio](https://github.com/awestlake87/pyo3-asyncio) Utilities for working with Python's Asyncio library and async functions

## Examples

- [hyperjson](https://github.com/mre/hyperjson) _A hyper-fast Python module for reading/writing JSON data using Rust's serde-json_
- [html-py-ever](https://github.com/PyO3/setuptools-rust/tree/main/examples/html-py-ever) _Using [html5ever](https://github.com/servo/html5ever) through [kuchiki](https://github.com/kuchiki-rs/kuchiki) to speed up html parsing and css-selecting._
- [point-process](https://github.com/ManifoldFR/point-process-rust/tree/master/pylib) _High level API for pointprocesses as a Python library_
- [autopy](https://github.com/autopilot-rs/autopy) _A simple, cross-platform GUI automation library for Python and Rust._
  - Contains an example of building wheels on TravisCI and appveyor using [cibuildwheel](https://github.com/joerick/cibuildwheel)
- [orjson](https://github.com/ijl/orjson) _Fast Python JSON library_
- [inline-python](https://github.com/dronesforwork/inline-python) _Inline Python code directly in your Rust code_
- [Rogue-Gym](https://github.com/kngwyu/rogue-gym) _Customizable rogue-like game for AI experiments_
  - Contains an example of building wheels on Azure Pipelines
- [fastuuid](https://github.com/thedrow/fastuuid/) _Python bindings to Rust's UUID library_
- [wasmer-python](https://github.com/wasmerio/wasmer-python) _Python library to run WebAssembly binaries_
- [mocpy](https://github.com/cds-astro/mocpy) _Astronomical Python library offering data structures for describing any arbitrary coverage regions on the unit sphere_
- [tokenizers](https://github.com/huggingface/tokenizers/tree/master/bindings/python) _Python bindings to the Hugging Face tokenizers (NLP) written in Rust_
- [pyre](https://github.com/Project-Dream-Weaver/Pyre) _Fast Python HTTP server written in Rust_
- [jsonschema-rs](https://github.com/Stranger6667/jsonschema-rs/tree/master/bindings/python) _Fast JSON Schema validation library_
- [css-inline](https://github.com/Stranger6667/css-inline/tree/master/bindings/python) _CSS inlining for Python implemented in Rust_
- [cryptography](https://github.com/pyca/cryptography/tree/main/src/rust) _Python cryptography library with some functionality in Rust_
- [polaroid](https://github.com/daggy1234/polaroid) _Hyper Fast and safe image manipulation library for Python written in Rust_
- [ormsgpack](https://github.com/aviramha/ormsgpack) _Fast Python msgpack library_

## Contributing

Everyone is welcomed to contribute to PyO3! There are many ways to support the project, such as:

- help PyO3 users with issues on Github and Gitter
- improve documentation
- write features and bugfixes
- publish blogs and examples of how to use PyO3

Our [contributing notes](https://github.com/PyO3/pyo3/blob/main/Contributing.md) and [architecture guide](https://github.com/PyO3/pyo3/blob/master/Architecture.md) have more resources if you wish to volunteer time for PyO3 and are searching where to start.

If you don't have time to contribute yourself but still wish to support the project's future success, some of our maintainers have Github sponsorship pages:

- [davidhewitt](https://github.com/sponsors/davidhewitt)

## License

PyO3 is licensed under the [Apache-2.0 license](http://opensource.org/licenses/APACHE-2.0).
Python is licensed under the [Python License](https://docs.python.org/2/license.html).
