# Building and Distribution

## Python version

PyO3 uses a build script to determine the Python version and set the correct linker arguments. By default it uses the `python3` executable. You can override the Python interpreter by setting `PYO3_PYTHON`, e.g., `PYO3_PYTHON=python3.6`.

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

There are two ways to distribute your module as a Python package: The old, [setuptools-rust], and the new, [maturin]. setuptools-rust needs several configuration files (`setup.py`, `MANIFEST.in`, `build-wheels.sh`, etc.). maturin doesn't need any configuration files, however it does not support some functionality of setuptools such as package data ([pyo3/maturin#258](https://github.com/PyO3/maturin/issues/258)) and requires a rigid project structure, while setuptools-rust allows (and sometimes requires) configuration with python code.

## `Py_LIMITED_API`/`abi3`

By default, Python extension modules can only be used with the same Python version they were compiled against -- if you build an extension module with Python 3.5, you can't import it using Python 3.8. [PEP 384](https://www.python.org/dev/peps/pep-0384/) introduced the idea of the limited Python API, which would have a stable ABI enabling extension modules built with it to be used against multiple Python versions. This is also known as `abi3`.

Note that [maturin] >= 0.9.0 or [setuptools-rust] >= 0.11.4 support `abi3` wheels.
See the [corresponding](https://github.com/PyO3/maturin/pull/353) [PRs](https://github.com/PyO3/setuptools-rust/pull/82) for more.

There are three steps involved in making use of `abi3` when building Python packages as wheels:

1. Enable the `abi3` feature in `pyo3`. This ensures `pyo3` only calls Python C-API functions which are part of the stable API, and on Windows also ensures that the project links against the correct shared object (no special behavior is required on other platforms):

```toml
[dependencies]
pyo3 = { version = "...", features = ["abi3"]}
```

2. Ensure that the built shared objects are correctly marked as `abi3`. This is accomplished by telling your build system that you're using the limited API.

3. Ensure that the `.whl` is correctly marked as `abi3`. For projects using `setuptools`, this is accomplished by passing `--py-limited-api=cp3x` (where `x` is the minimum Python version supported by the wheel, e.g. `--py-limited-api=cp35` for Python 3.5) to `setup.py bdist_wheel`.

### Minimum Python version for `abi3`

Because a single `abi3` wheel can be used with many different Python versions, PyO3 has feature flags `abi3-py36`, `abi3-py37`, `abi-py38` etc. to set the minimum required Python version for your `abi3` wheel.
For example, if you set the `abi3-py36` feature, your extension wheel can be used on all Python 3 versions from Python 3.6 and up. `maturin` and `setuptools-rust` will give the wheel a name like `my-extension-1.0-cp36-abi3-manylinux2020_x86_64.whl`.
If you set more that one of these api version feature flags the highest version always wins. For example, with both `abi3-py36` and `abi3-py38` set, PyO3 would build a wheel which supports Python 3.8 and up.
PyO3 is only able to link your extension module to api3 version up to and including your host Python version. E.g., if you set `abi3-py38` and try to compile the crate with a host of Python 3.6, the build will fail.

As an advanced feature, you can build PyO3 wheel without calling Python interpreter with the environment variable `PYO3_NO_PYTHON` set. On unix systems this works unconditionally; on Windows you must also set the `RUSTFLAGS` evironment variable to contain `-L native=/path/to/python/libs` so that the linker can find `python3.lib`.

### Missing features

Due to limitations in the Python API, there are a few `pyo3` features that do
not work when compiling for `abi3`. These are:

- `#[text_signature]` does not work on classes until Python 3.10 or greater.
- The `dict` and `weakref` options on classes are not supported until Python 3.9 or greater.
- The buffer API is not supported.

## Cross Compiling

Cross compiling PyO3 modules is relatively straightforward and requires a few pieces of software:

* A toolchain for your target.
* The appropriate options in your Cargo `.config` for the platform you're targeting and the toolchain you are using.
* A Python interpreter that's already been compiled for your target.
* A Python interpreter that is built for your host and available through the `PATH` or setting the [`PYO3_PYTHON`](#python-version) variable.
* The headers that match the above interpreter.

See https://github.com/japaric/rust-cross for a primer on cross compiling Rust in general.

After you've obtained the above, you can build a cross compiled PyO3 module by setting a few extra environment variables:

* `PYO3_CROSS_LIB_DIR`: This variable must be set to the directory containing the target's libpython DSO and the associated `_sysconfigdata*.py` file.
* `PYO3_CROSS_PYTHON_VERSION`: Major and minor version (e.g. 3.9) of the target Python installation. This variable is only needed if pyo3 cannot determine the version to target by other means:
  - From `PYO3_CROSS_INCLUDE_DIR` or abi3-py3* features when targeting Windows, or
  - if there are multiple versions of python present in `PYO3_CROSS_LIB_DIR` when targeting unix.
* `PYO3_CROSS_INCLUDE_DIR`: This variable can optionally be set to the directory containing the headers for the target's Python interpreter when targeting Windows.

An example might look like the following (assuming your target's sysroot is at `/home/pyo3/cross/sysroot` and that your target is `armv7`):

```sh
export PYO3_CROSS_LIB_DIR="/home/pyo3/cross/sysroot/usr/lib"

cargo build --target armv7-unknown-linux-gnueabihf
```

If there are multiple python versions at the cross lib directory and you cannot set a more precise location to include both the `libpython` DSO and `_sysconfigdata*.py` files, you can set the required version:
```sh
export PYO3_CROSS_PYTHON_VERSION=3.8
export PYO3_CROSS_LIB_DIR="/home/pyo3/cross/sysroot/usr/lib"

cargo build --target armv7-unknown-linux-gnueabihf
```

Or another example with the same sys root but building for windows:
```sh
export PYO3_CROSS_INCLUDE_DIR="/home/pyo3/cross/sysroot/usr/include"
export PYO3_CROSS_LIB_DIR="/home/pyo3/cross/sysroot/usr/lib"

cargo build --target x86_64-pc-windows-gnu
```

## Bazel

For an example of how to build python extensions using Bazel, see https://github.com/TheButlah/rules_pyo3


[maturin]: https://github.com/PyO3/maturin
[setuptools-rust]: https://github.com/PyO3/setuptools-rust
