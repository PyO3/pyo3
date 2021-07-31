# Building and Distribution

## Python version

PyO3 uses a build script to determine the Python version and set the correct linker arguments. By default it uses the `python3` executable. You can override the Python interpreter by setting `PYO3_PYTHON`, e.g., `PYO3_PYTHON=python3.6`.

## Linking

Different linker arguments must be set for libraries/extension modules and binaries, which includes both standalone binaries and tests. (More specifically, binaries must be told where to find libpython and libraries must not link to libpython for [manylinux](https://www.python.org/dev/peps/pep-0513/) compliance).

Since PyO3's build script can't know whether you're building a binary or a library, you have to activate the `extension-module` feature to get the build options for a library, or it'll default to binary.

If you have e.g. a library crate and a profiling crate alongside, you need to use optional features. E.g. you put the following in the library crate:

```toml
[dependencies]
pyo3 = { {{#PYO3_CRATE_VERSION}} }


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
pyo3 = { {{#PYO3_CRATE_VERSION}} }
```

On Linux/macOS you might have to change `LD_LIBRARY_PATH` to include libpython, while on windows you might need to set `LIB` to include `pythonxy.lib` (where x and y are major and minor version), which is normally either in the `libs` or `Lib` folder of a Python installation.

## Testing, building and distribution

There are two main ways to test, build and distribute your module as a Python package: [setuptools-rust] and [maturin]. setuptools-rust needs several configuration files (`setup.py`, `MANIFEST.in`, `build-wheels.sh`, etc.). It allows (and sometimes requires) writing custom workflows in python. maturin has only few options and works without any additional configuration, instead it requires a rigid project structure and does not support some functionality of setuptools such as package data ([pyo3/maturin#258](https://github.com/PyO3/maturin/issues/258)), multiple extensions or running python scripts at build time.

### Manual builds

You can also symlink (or copy) and rename the shared library from the `target` folder:
- on macOS, rename `libyour_module.dylib` to `your_module.so`.
- on Windows, rename  `libyour_module.dll` to `your_module.pyd`.
- on Linux, rename `libyour_module.so` to `your_module.so`.

You can then open a Python shell in the same folder and you'll be able to run `import your_module`.

## `Py_LIMITED_API`/`abi3`

By default, Python extension modules can only be used with the same Python version they were compiled against -- if you build an extension module with Python 3.5, you can't import it using Python 3.8. [PEP 384](https://www.python.org/dev/peps/pep-0384/) introduced the idea of the limited Python API, which would have a stable ABI enabling extension modules built with it to be used against multiple Python versions. This is also known as `abi3`.

The advantage of building extension module using the limited Python API is that you only need to build and distribute a single copy (for each OS / architecture), and your users can install it on all Python versions from your [minimum version](#minimum-python-version-for-abi3) and up. The downside of this is that PyO3 can't use optimizations which rely on being compiled against a known exact Python version. It's up to you to decide whether this matters for your extension module. It's also possible to design your extension module such that you can distribute `abi3` wheels but allow users compiling from source to benefit from additional optimizations - see the [support for multiple python versions](./building_and_distribution/multiple_python_versions.html) section of this guide, in particular the `#[cfg(Py_LIMITED_API)]` flag.

There are three steps involved in making use of `abi3` when building Python packages as wheels:

1. Enable the `abi3` feature in `pyo3`. This ensures `pyo3` only calls Python C-API functions which are part of the stable API, and on Windows also ensures that the project links against the correct shared object (no special behavior is required on other platforms):

```toml
[dependencies]
pyo3 = { {{#PYO3_CRATE_VERSION}}, features = ["abi3"] }
```

2. Ensure that the built shared objects are correctly marked as `abi3`. This is accomplished by telling your build system that you're using the limited API. [maturin] >= 0.9.0 and [setuptools-rust] >= 0.11.4 support `abi3` wheels.
See the [corresponding](https://github.com/PyO3/maturin/pull/353) [PRs](https://github.com/PyO3/setuptools-rust/pull/82) for more.

3. Ensure that the `.whl` is correctly marked as `abi3`. For projects using `setuptools`, this is accomplished by passing `--py-limited-api=cp3x` (where `x` is the minimum Python version supported by the wheel, e.g. `--py-limited-api=cp35` for Python 3.5) to `setup.py bdist_wheel`.

### Minimum Python version for `abi3`

Because a single `abi3` wheel can be used with many different Python versions, PyO3 has feature flags `abi3-py36`, `abi3-py37`, `abi-py38` etc. to set the minimum required Python version for your `abi3` wheel.
For example, if you set the `abi3-py36` feature, your extension wheel can be used on all Python 3 versions from Python 3.6 and up. `maturin` and `setuptools-rust` will give the wheel a name like `my-extension-1.0-cp36-abi3-manylinux2020_x86_64.whl`.

As your extension module may be run with multiple different Python versions you may occasionally find you need to check the Python version at runtime to customize behavior. See [the relevant section of this guide](./building_and_distribution/multiple_python_versions.html#checking-the-python-version-at-runtime) on supporting multiple Python versions at runtime.

PyO3 is only able to link your extension module to api3 version up to and including your host Python version. E.g., if you set `abi3-py38` and try to compile the crate with a host of Python 3.6, the build will fail.

As an advanced feature, you can build PyO3 wheel without calling Python interpreter with the environment variable `PYO3_NO_PYTHON` set. On unix systems this works unconditionally; on Windows you must also set the `RUSTFLAGS` evironment variable to contain `-L native=/path/to/python/libs` so that the linker can find `python3.lib`.

> Note: If you set more that one of these api version feature flags the highest version always wins. For example, with both `abi3-py36` and `abi3-py38` set, PyO3 would build a wheel which supports Python 3.8 and up.

### Missing features

Due to limitations in the Python API, there are a few `pyo3` features that do
not work when compiling for `abi3`. These are:

- `#[pyo3(text_signature = "...")]` does not work on classes until Python 3.10 or greater.
- The `dict` and `weakref` options on classes are not supported until Python 3.9 or greater.
- The buffer API is not supported.
- Optimizations which rely on knowledge of the exact Python version compiled against.

## Cross Compiling

Cross compiling PyO3 modules is relatively straightforward and requires a few pieces of software:

* A toolchain for your target.
* The appropriate options in your Cargo `.config` for the platform you're targeting and the toolchain you are using.
* A Python interpreter that's already been compiled for your target.
* A Python interpreter that is built for your host and available through the `PATH` or setting the [`PYO3_PYTHON`](#python-version) variable.

See [github.com/japaric/rust-cross](https://github.com/japaric/rust-cross) for a primer on cross compiling Rust in general.

After you've obtained the above, you can build a cross compiled PyO3 module by setting a few extra environment variables:

* `PYO3_CROSS`: If present this variable forces PyO3 to configure as a cross-compilation.
* `PYO3_CROSS_LIB_DIR`: This variable must be set to the directory containing the target's libpython DSO and the associated `_sysconfigdata*.py` file for Unix-like targets, or the Python DLL import libraries for the Windows target.
* `PYO3_CROSS_PYTHON_VERSION`: Major and minor version (e.g. 3.9) of the target Python installation. This variable is only needed if PyO3 cannot determine the version to target from `abi3-py3*` features, or if there are multiple versions of Python present in `PYO3_CROSS_LIB_DIR`.

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

Or another example with the same sys root but building for Windows:
```sh
export PYO3_CROSS_PYTHON_VERSION=3.9
export PYO3_CROSS_LIB_DIR="/home/pyo3/cross/sysroot/usr/lib"

cargo build --target x86_64-pc-windows-gnu
```

Any of the `abi3-py3*` features can be enabled instead of setting `PYO3_CROSS_PYTHON_VERSION` in the above examples.

## Embedding Python in Rust

If you want to embed the Python interpreter inside a Rust program, there are two modes in which this can be done: dynamically and statically. We'll cover each of these modes in the following sections. Each of them affect how you must distribute your program. Instead of learning how to do this yourself, you might want to consider using a project like [PyOxidizer] to ship your application and all of its dependencies in a single file.

PyO3 automatically switches between the two linking modes depending on whether the Python distribution you have configured PyO3 to use ([see above](#python-version)) contains a shared library or a static library. The static library is most often seen in Python distributions compiled from source without the `--enable-shared` configuration option. For example, this is the default for `pyenv` on macOS.

### Dynamically embedding the Python interpreter

Embedding the Python interpreter dynamically is much easier than doing so statically. This is done by linking your program against a Python shared library (such as `libpython.3.9.so` on UNIX, or `python39.dll` on Windows). The implementation of the Python interpreter resides inside the shared library. This means that when the OS runs your Rust program it also needs to be able to find the Python shared library.

This mode of embedding works well for Rust tests which need access to the Python interpreter. It is also great for Rust software which is installed inside a Python virtualenv, because the virtualenv sets up appropriate environment variables to locate the correct Python shared library.

For distributing your program to non-technical users, you will have to consider including the Python shared library in your distribution as well as setting up wrapper scripts to set the right environment variables (such as `LD_LIBRARY_PATH` on UNIX, or `PATH` on Windows).

### Statically embedding the Python interpreter

Embedding the Python interpreter statically means including the contents of a Python static library directly inside your Rust binary. This means that to distribute your program you only need to ship your binary file: it contains the Python interpreter inside the binary!

On Windows static linking is almost never done, so Python distributions don't usually include a static library. The information below applies only to UNIX.

The Python static library is usually called `libpython.a`.

Static linking has a lot of complications, listed below. For these reasons PyO3 does not yet have first-class support for this embedding mode. See [issue 416 on PyO3's Github](https://github.com/PyO3/pyo3/issues/416) for more information and to discuss any issues you encounter.

The [`auto-initialize`](features.md#auto-initialize) feature is deliberately disabled when embedding the interpreter statically because this is often unintentionally done by new users to PyO3 running test programs. Trying out PyO3 is much easier using dynamic embedding.

The known complications are:
  - To import compiled extension modules (such as other Rust extension modules, or those written in C), your binary must have the correct linker flags set during compilation to export the original contents of `libpython.a` so that extensions can use them (e.g. `-Wl,--export-dynamic`).
  - The C compiler and flags which were used to create `libpython.a` must be compatible with your Rust compiler and flags, else you will experience compilation failures.

    Significantly different compiler versions may see errors like this:

    ```ignore
    lto1: fatal error: bytecode stream in file 'rust-numpy/target/release/deps/libpyo3-6a7fb2ed970dbf26.rlib' generated with LTO version 6.0 instead of the expected 6.2
    ```

    Mismatching flags may lead to errors like this:

    ```ignore
    /usr/bin/ld: /usr/lib/gcc/x86_64-linux-gnu/9/../../../x86_64-linux-gnu/libpython3.9.a(zlibmodule.o): relocation R_X86_64_32 against `.data' can not be used when making a PIE object; recompile with -fPIE
    ```

If you encounter these or other complications when linking the interpreter statically, discuss them on [issue 416 on PyO3's Github](https://github.com/PyO3/pyo3/issues/416). It is hoped that eventually that discussion will contain enough information and solutions that PyO3 can offer first-class support for static embedding.

## Bazel

For an example of how to build python extensions using Bazel, see https://github.com/TheButlah/rules_pyo3


[maturin]: https://github.com/PyO3/maturin
[setuptools-rust]: https://github.com/PyO3/setuptools-rust
[PyOxidizer]: https://github.com/indygreg/PyOxidizer
