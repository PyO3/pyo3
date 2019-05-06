# Building and Distribution

## Python version

PyO3 uses a build script to determine the Python version and set the correct linker arguments. By default it uses the `python3` executable. You can override the Python interpreter by setting `PYTHON_SYS_EXECUTABLE`, e.g., `PYTHON_SYS_EXECUTABLE=python3.6`.

## Linking

Different linker arguments must be set for libraries/extension modules and binaries, which includes both standalone binaries and tests. (More specifically, binaries must be told where to find libpython and libraries must not link to libpython for [manylinux](https://www.python.org/dev/peps/pep-0513/) compliance).

Since PyO3's build script can't know whether you're building a binary or a library, you have to activate the `extension-module` feature to get the build options for a library, or it'll default to binary.

If you have e.g. a library crate and a profiling crate alongside, you need to use optional features. E.g. you put the following in the library crate:

```toml
[dependencies]
pyo3 = "0.6"

[lib]
name = "hyperjson"
crate-type = ["rlib", "cdylib"]

[features]
default = ["pyo3/extension-module"]
```

And this in the profiling crate:

```toml
[dependencies]
my_main_crate = { path = "..", default-features = false }
pyo3 = "0.6"
```

On Linux/macOS you might have to change `LD_LIBRARY_PATH` to include libpython, while on windows you might need to set `LIB` to include `pythonxy.lib` (where x and y are major and minor version), which is normally either in the `libs` or `Lib` folder of a Python installation.

## Distribution

There are two ways to distribute your module as a Python package: the old, [setuptools-rust](https://github.com/PyO3/setuptools-rust), and the new, [pyo3-pack](https://github.com/pyo3/pyo3-pack). setuptools-rust needs some configuration files (`setup.py`, `MANIFEST.in`, `build-wheels.sh`, etc.) and external tools (docker, twine). pyo3-pack doesn't need any configuration files. It can not yet build sdist though ([pyo3/pyo3-pack#2](https://github.com/PyO3/pyo3-pack/issues/2)).

## Cross Compiling

Cross compiling PyO3 modules is relatively straightforward and requires a few pieces of software:

* A toolchain for your target.
* The appropriate options in your Cargo `.config` for the platform you're targeting and the toolchain you are using.
* A Python interpreter that's already been compiled for your target.
* The headers that match the above interpreter.

See https://github.com/japaric/rust-cross for a primer on cross compiling Rust in general.

After you've obtained the above, you can build a cross compiled PyO3 module by setting a few extra environment variables:

* `PYO3_CROSS_INCLUDE_DIR`: This variable must be set to the directory containing the headers for the target's Python interpreter.
* `PYO3_CROSS_LIB_DIR`: This variable must be set to the directory containing the target's libpython DSO.

An example might look like the following (assuming your target's sysroot is at `/home/pyo3/cross/sysroot` and that your target is `armv7`):

```sh
export PYO3_CROSS_INCLUDE_DIR="/home/pyo3/cross/sysroot/usr/include"
export PYO3_CROSS_LIB_DIR="/home/pyo3/cross/sysroot/usr/lib"

cargo build --target armv7-unknown-linux-gnueabihf
```
