# Installation

To get started using PyO3 you will need three things: a Rust toolchain, a Python environment, and a way to build. We'll cover each of these below.

> If you'd like to chat to the PyO3 maintainers and other PyO3 users, consider joining the [PyO3 Discord server](https://discord.gg/33kcChzH7f). We're keen to hear about your experience getting started, so we can make PyO3 as accessible as possible for everyone!

## Rust

First, make sure you have Rust installed on your system. If you haven't already done so, try following the instructions [here](https://www.rust-lang.org/tools/install). PyO3 runs on both the `stable` and `nightly` versions so you can choose whichever one fits you best. The minimum required Rust version is 1.74.

If you can run `rustc --version` and the version is new enough you're good to go!

## Python

To use PyO3, you need at least Python 3.7. While you can simply use the default Python interpreter on your system, it is recommended to use a virtual environment.

## Virtualenvs

While you can use any virtualenv manager you like, we recommend the use of `pyenv` in particular if you want to develop or test for multiple different Python versions, so that is what the examples in this book will use. The installation instructions for `pyenv` can be found [here](https://github.com/pyenv/pyenv#a-getting-pyenv). (Note: To get the `pyenv activate` and `pyenv virtualenv` commands, you will also need to install the [`pyenv-virtualenv`](https://github.com/pyenv/pyenv-virtualenv) plugin. The [pyenv installer](https://github.com/pyenv/pyenv-installer#installation--update--uninstallation) will install both together.)

It can be useful to keep the sources used when installing using `pyenv` so that future debugging can see the original source files. This can be done by passing the `--keep` flag as part of the `pyenv install` command.

For example:

```bash
pyenv install 3.12 --keep
```

### Building

There are a number of build and Python package management systems such as [`setuptools-rust`](https://github.com/PyO3/setuptools-rust) or [manually](./building-and-distribution.md#manual-builds). We recommend the use of `maturin`, which you can install [here](https://maturin.rs/installation.html). It is developed to work with PyO3 and provides the most "batteries included" experience, especially if you are aiming to publish to PyPI. `maturin` is just a Python package, so you can add it in the same way you already install Python packages.

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
poetry add -G dev maturin
```

After installation, you can run `maturin --version` to check that you have correctly installed it.

# Starting a new project

First you should create the folder and virtual environment that are going to contain your new project. Here we will use the recommended `pyenv`:

```bash
mkdir pyo3-example
cd pyo3-example
pyenv virtualenv pyo3
pyenv local pyo3
```

After this, you should install your build manager. In this example, we will use `maturin`. After you've activated your virtualenv, add `maturin` to it:

```bash
pip install maturin
```

Now you can initialize the new project:

```bash
maturin init
```

If `maturin` is already installed, you can create a new project using that directly as well:

```bash
maturin new -b pyo3 pyo3-example
cd pyo3-example
pyenv virtualenv pyo3
pyenv local pyo3
```

# Adding to an existing project

Sadly, `maturin` cannot currently be run in existing projects, so if you want to use Python in an existing project you basically have two options:

1. Create a new project as above and move your existing code into that project
2. Manually edit your project configuration as necessary

If you opt for the second option, here are the things you need to pay attention to:

## Cargo.toml

Make sure that the Rust crate you want to be able to access from Python is compiled into a library. You can have a binary output as well, but the code you want to access from Python has to be in the library part. Also, make sure that the crate type is `cdylib` and add PyO3 as a dependency as so:


```toml
# If you already have [package] information in `Cargo.toml`, you can ignore
# this section!
[package]
# `name` here is name of the package.
name = "pyo3_start"
# these are good defaults:
version = "0.1.0"
edition = "2021"

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
requires = ["maturin>=1,<2"]
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

After this you can setup Rust code to be available in Python as below; for example, you can place this code in `src/lib.rs`:

```rust,no_run
/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pyo3::pymodule]
mod pyo3_example {
    use pyo3::prelude::*;

    /// Formats the sum of two numbers as string.
    #[pyfunction]
    fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
        Ok((a + b).to_string())
    }
}
```

Now you can run `maturin develop` to prepare the Python package, after which you can use it like so:

```bash
$ maturin develop
# lots of progress output as maturin runs the compilation...
$ python
>>> import pyo3_example
>>> pyo3_example.sum_as_string(5, 20)
'25'
```

For more instructions on how to use Python code from Rust, see the [Python from Rust](python-from-rust.md) page.

## Maturin Import Hook

In development, any changes in the code would require running `maturin develop` before testing. To streamline the development process, you may want to install [Maturin Import Hook](https://github.com/PyO3/maturin-import-hook) which will run `maturin develop` automatically when the library with code changes is being imported.
