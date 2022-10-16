
# Installation

To get started using PyO3 you will need three things: a rust toolchain, a python environment, and a way to build. We'll cover each of these below.

## Rust

First, make sure you have rust installed on your system. If you haven't already done so you can do so by following the instructions [here](https://www.rust-lang.org/tools/install). PyO3 runs on both the `stable` and `nightly` versions so you can choose whichever one fits you best. The minimum required rust version is Rust 1.48.

if you can run `rustc --version` and the version is high enough you're good to go!

## Python

To use PyO3 you need at least Python 3.7. While you can simply use the default Python version on your system, it is recommended to use a virtual environment.

## Virtualenvs

While you can use any virtualenv manager you like, we recommend the use of `pyenv` especially if you want to develop or test for multiple different python versions, so that is what the examples in this book will use. The installation instructions for `pyenv` can be found [here](https://github.com/pyenv/pyenv#getting-pyenv).

Note that when using `pyenv` you should also set the following environment variable
```bash
PYTHON_CONFIGURE_OPTS="--enable-shared"
```

### Building

There are a number of build and python package management systems such as [`setuptools-rust`](https://github.com/PyO3/setuptools-rust) or [manually](https://pyo3.rs/latest/building_and_distribution.html#manual-builds)  we recommend the use of `maturin` which you can install [here](https://maturin.rs/installation.html). It is developed to work with PyO3 and is the most "batteries included" experience. `maturin` is just a python package so you can add it in any way that you install python packages.

System Python:
```bash
pip install maturin --user
```

pipx:
```bash
pipx install maturin
```

pyenv:
```bash
pyenv activate pyo3
pip install maturin
```

poetry:
```bash
poetry add -D maturin
```

after installation, you can run `maturin --version` to check that you have correctly installed it.

# Starting a new project

Firstly you should create the folder and virtual environment that are going to contain your new project. Here we will use the recommended `pyenv`:

```bash
mkdir pyo3-example
cd pyo3-example
pyenv virtualenv pyo3
pyenv local pyo3
```
after this, you should install your build manager. In this example, we will use `maturin`. After you've activated your virtualenv add `maturin` to it:

```bash
pip install maturin
```

After this, you can initialise the new project

```bash
maturin init
```

If `maturin` is already installed you can create a new project using that directly as well:

```bash
maturin new -b pyo3 pyo3-example
cd pyo3-example
pyenv virtualenv pyo3
pyenv local pyo3
```

# Adding to an existing project

Sadly currently `maturin` cannot be run in existing projects, so if you want to use python in an existing project you basically have two options:

1. create a new project as above and move your existing code into that project
2. Manually edit your project configuration as necessary.

If you are opting for the second option, here are the things you need to pay attention to:

## Cargo.toml

Make sure that the rust you want to be able to access from Python is compiled into a library. You can have a binary output as well, but the code you want to access from python has to be in the library. Also, make sure that the crate type is `cdylib`  and add PyO3 as a dependency as so:


```toml
[lib]
# The name of the native library. This is the name which will be used in Python to import the
# library (i.e. `import string_sum`). If you change this, you must also change the name of the
# `#[pymodule]` in `src/lib.rs`.
name = "pyo3_example"

# "cdylib" is necessary to produce a shared library for Python to import from.
crate-type = ["cdylib"]

[dependencies]
pyo3 = { {{#PYO3_CRATE_VERSION}}, features = ["extension-module"] }
```

## pyproject.toml

You should also create a `pyproject.toml` with the following contents:

```toml
[build-system]
requires = ["maturin>=0.13,<0.14"]
build-backend = "maturin"

[project]
name = "pyo3_example"
requires-python = ">=3.7"
classifiers = [
    "Programming Language :: Rust",
    "Programming Language :: Python :: Implementation :: CPython",
    "Programming Language :: Python :: Implementation :: PyPy",
]
```

## Running code

After this you can setup rust code to be available in python as below; for example, you can place this code in `src/lib.rs`

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
fn string_sum(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(sum_as_string, m)?)?;
    Ok(())
}
```

After this you can run `maturin develop` to prepare the python package after which you can use it like so:

```bash
$ maturin develop
# lots of progress output as maturin runs the compilation...
$ python
>>> import pyo3_example
>>> pyo3_example.sum_as_string(5, 20)
'25'
```

For more instructions on how to use python code from rust see the [Python from Rust](python_from_rust.md) page.
