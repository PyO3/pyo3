# Building and distribution

This chapter of the guide goes into detail on how to build and distribute projects using PyO3. The way to achieve this is very different depending on whether the project is a Python module implemented in Rust, or a Rust binary embedding Python. For both types of project there are also common problems such as the Python version to build for and the [linker](https://en.wikipedia.org/wiki/Linker_(computing)) arguments to use.

The material in this chapter is intended for users who have already read the PyO3 [README](./index.md). It covers in turn the choices that can be made for Python modules and for Rust binaries. There is also a section at the end about cross-compiling projects using PyO3.

There is an additional sub-chapter dedicated to [supporting multiple Python versions](./building-and-distribution/multiple-python-versions.md).

## Configuring the Python version

PyO3 uses a build script (backed by the [`pyo3-build-config`] crate) to determine the Python version and set the correct linker arguments. By default it will attempt to use the following in order:
 - Any active Python virtualenv.
 - The `python` executable (if it's a Python 3 interpreter).
 - The `python3` executable.

You can override the Python interpreter by setting the `PYO3_PYTHON` environment variable, e.g. `PYO3_PYTHON=python3.7`, `PYO3_PYTHON=/usr/bin/python3.9`, or even a PyPy interpreter `PYO3_PYTHON=pypy3`.

Once the Python interpreter is located, `pyo3-build-config` executes it to query the information in the `sysconfig` module which is needed to configure the rest of the compilation.

To validate the configuration which PyO3 will use, you can run a compilation with the environment variable `PYO3_PRINT_CONFIG=1` set. An example output of doing this is shown below:

```console
$ PYO3_PRINT_CONFIG=1 cargo build
   Compiling pyo3 v0.14.1 (/home/david/dev/pyo3)
error: failed to run custom build command for `pyo3 v0.14.1 (/home/david/dev/pyo3)`

Caused by:
  process didn't exit successfully: `/home/david/dev/pyo3/target/debug/build/pyo3-7a8cf4fe22e959b7/build-script-build` (exit status: 101)
  --- stdout
  cargo:rerun-if-env-changed=PYO3_CROSS
  cargo:rerun-if-env-changed=PYO3_CROSS_LIB_DIR
  cargo:rerun-if-env-changed=PYO3_CROSS_PYTHON_VERSION
  cargo:rerun-if-env-changed=PYO3_PRINT_CONFIG

  -- PYO3_PRINT_CONFIG=1 is set, printing configuration and halting compile --
  implementation=CPython
  version=3.8
  shared=true
  abi3=false
  lib_name=python3.8
  lib_dir=/usr/lib
  executable=/usr/bin/python
  pointer_width=64
  build_flags=
  suppress_build_script_link_lines=false
```

The `PYO3_ENVIRONMENT_SIGNATURE` environment variable can be used to trigger rebuilds when its value changes, it has no other effect.

### Advanced: config files

If you save the above output config from `PYO3_PRINT_CONFIG` to a file, it is possible to manually override the contents and feed it back into PyO3 using the `PYO3_CONFIG_FILE` env var.

If your build environment is unusual enough that PyO3's regular configuration detection doesn't work, using a config file like this will give you the flexibility to make PyO3 work for you. To see the full set of options supported, see the documentation for the [`InterpreterConfig` struct](https://docs.rs/pyo3-build-config/{{#PYO3_DOCS_VERSION}}/pyo3_build_config/struct.InterpreterConfig.html).

## Building Python extension modules

Python extension modules need to be compiled differently depending on the OS (and architecture) that they are being compiled for. As well as multiple OSes (and architectures), there are also many different Python versions which are actively supported. Packages uploaded to [PyPI](https://pypi.org/) usually want to upload prebuilt "wheels" covering many OS/arch/version combinations so that users on all these different platforms don't have to compile the package themselves. Package vendors can opt-in to the "abi3" limited Python API which allows their wheels to be used on multiple Python versions, reducing the number of wheels they need to compile, but restricts the functionality they can use.

There are many ways to go about this: it is possible to use `cargo` to build the extension module (along with some manual work, which varies with OS). The PyO3 ecosystem has two packaging tools, [`maturin`] and [`setuptools-rust`], which abstract over the OS difference and also support building wheels for PyPI upload.

PyO3 has some Cargo features to configure projects for building Python extension modules:
 - The `extension-module` feature, which must be enabled when building Python extension modules.
 - The `abi3` feature and its version-specific `abi3-pyXY` companions, which are used to opt-in to the limited Python API in order to support multiple Python versions in a single wheel.

This section describes each of these packaging tools before describing how to build manually without them. It then proceeds with an explanation of the `extension-module` feature. Finally, there is a section describing PyO3's `abi3` features.

### Packaging tools

The PyO3 ecosystem has two main choices to abstract the process of developing Python extension modules:
- [`maturin`] is a command-line tool to build, package and upload Python modules. It makes opinionated choices about project layout meaning it needs very little configuration. This makes it a great choice for users who are building a Python extension from scratch and don't need flexibility.
- [`setuptools-rust`] is an add-on for `setuptools` which adds extra keyword arguments to the `setup.py` configuration file. It requires more configuration than `maturin`, however this gives additional flexibility for users adding Rust to an existing Python package that can't satisfy `maturin`'s constraints.

Consult each project's documentation for full details on how to get started using them and how to upload wheels to PyPI. It should be noted that while `maturin` is able to build [manylinux](https://github.com/pypa/manylinux)-compliant wheels out-of-the-box, `setuptools-rust` requires a bit more effort, [relying on Docker](https://setuptools-rust.readthedocs.io/en/latest/building_wheels.html) for this purpose.

There are also [`maturin-starter`] and [`setuptools-rust-starter`] examples in the PyO3 repository.

### Manual builds

To build a PyO3-based Python extension manually, start by running `cargo build` as normal in a library project which uses PyO3's `extension-module` feature and has the [`cdylib` crate type](https://doc.rust-lang.org/cargo/reference/cargo-targets.html#the-crate-type-field).

Once built, symlink (or copy) and rename the shared library from Cargo's `target/` directory to your desired output directory:
- on macOS, rename `libyour_module.dylib` to `your_module.so`.
- on Windows, rename  `libyour_module.dll` to `your_module.pyd`.
- on Linux, rename `libyour_module.so` to `your_module.so`.

You can then open a Python shell in the output directory and you'll be able to run `import your_module`.

If you're packaging your library for redistribution, you should indicate the Python interpreter your library is compiled for by including the [platform tag](#platform-tags) in its name. This prevents incompatible interpreters from trying to import your library. If you're compiling for PyPy you *must* include the platform tag, or PyPy will ignore the module.

#### Bazel builds

To use PyO3 with bazel one needs to manually configure PyO3, PyO3-ffi and PyO3-macros. In particular, one needs to make sure that it is compiled with the right python flags for the version you intend to use.
For example see:

1. [github.com/abrisco/rules_pyo3](https://github.com/abrisco/rules_pyo3) -- General rules for building extension modules.
2. [github.com/OliverFM/pytorch_with_gazelle](https://github.com/OliverFM/pytorch_with_gazelle) -- for a minimal example of a repo that can use PyO3, PyTorch and Gazelle to generate python Build files.
3. [github.com/TheButlah/rules_pyo3](https://github.com/TheButlah/rules_pyo3) -- is somewhat dated.

#### Platform tags

Rather than using just the `.so` or `.pyd` extension suggested above (depending on OS), you can prefix the shared library extension with a platform tag to indicate the interpreter it is compatible with. You can query your interpreter's platform tag from the `sysconfig` module. Some example outputs of this are seen below:

```bash
# CPython 3.10 on macOS
.cpython-310-darwin.so

# PyPy 7.3 (Python 3.9) on Linux
$ python -c 'import sysconfig; print(sysconfig.get_config_var("EXT_SUFFIX"))'
.pypy39-pp73-x86_64-linux-gnu.so
```

So, for example, a valid module library name on CPython 3.10 for macOS is `your_module.cpython-310-darwin.so`, and its equivalent when compiled for PyPy 7.3 on Linux would be `your_module.pypy38-pp73-x86_64-linux-gnu.so`.

See [PEP 3149](https://peps.python.org/pep-3149/) for more background on platform tags.

#### macOS

On macOS, because the `extension-module` feature disables linking to `libpython` ([see the next section](#the-extension-module-feature)), some additional linker arguments need to be set. `maturin` and `setuptools-rust` both pass these arguments for PyO3 automatically, but projects using manual builds will need to set these directly in order to support macOS.

The easiest way to set the correct linker arguments is to add a [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html) with the following content:

```rust,ignore
fn main() {
    pyo3_build_config::add_extension_module_link_args();
}
```

Remember to also add `pyo3-build-config` to the `build-dependencies` section in `Cargo.toml`.

An alternative to using `pyo3-build-config` is add the following to a cargo configuration file (e.g. `.cargo/config.toml`):

```toml
[target.x86_64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]

[target.aarch64-apple-darwin]
rustflags = [
  "-C", "link-arg=-undefined",
  "-C", "link-arg=dynamic_lookup",
]
```

Using the MacOS system python3 (`/usr/bin/python3`, as opposed to python installed via homebrew, pyenv, nix, etc.) may result in runtime errors such as `Library not loaded: @rpath/Python3.framework/Versions/3.8/Python3`.

The easiest way to set the correct linker arguments is to add a `build.rs` with the following content:

```rust,ignore
fn main() {
    pyo3_build_config::add_python_framework_link_args();
}
```

Alternatively it can be resolved with another addition to `.cargo/config.toml`:

```toml
[build]
rustflags = [
  "-C", "link-args=-Wl,-rpath,/Library/Developer/CommandLineTools/Library/Frameworks",
]
```

For more discussion on and workarounds for MacOS linking problems [see this issue](https://github.com/PyO3/pyo3/issues/1800#issuecomment-906786649).

Finally, don't forget that on MacOS the `extension-module` feature will cause `cargo test` to fail without the `--no-default-features` flag (see [the FAQ](https://pyo3.rs/main/faq.html#i-cant-run-cargo-test-or-i-cant-build-in-a-cargo-workspace-im-having-linker-issues-like-symbol-not-found-or-undefined-reference-to-_pyexc_systemerror)).

### The `extension-module` feature

PyO3's `extension-module` feature is used to disable [linking](https://en.wikipedia.org/wiki/Linker_(computing)) to `libpython` on Unix targets.

This is necessary because by default PyO3 links to `libpython`. This makes binaries, tests, and examples "just work". However, Python extensions on Unix must not link to libpython for [manylinux](https://www.python.org/dev/peps/pep-0513/) compliance.

The downside of not linking to `libpython` is that binaries, tests, and examples (which usually embed Python) will fail to build. If you have an extension module as well as other outputs in a single project, you need to use optional Cargo features to disable the `extension-module` when you're not building the extension module. See [the FAQ](faq.md#i-cant-run-cargo-test-or-i-cant-build-in-a-cargo-workspace-im-having-linker-issues-like-symbol-not-found-or-undefined-reference-to-_pyexc_systemerror) for an example workaround.

### `Py_LIMITED_API`/`abi3`

By default, Python extension modules can only be used with the same Python version they were compiled against. For example, an extension module built for Python 3.5 can't be imported in Python 3.8. [PEP 384](https://www.python.org/dev/peps/pep-0384/) introduced the idea of the limited Python API, which would have a stable ABI enabling extension modules built with it to be used against multiple Python versions. This is also known as `abi3`.

The advantage of building extension modules using the limited Python API is that package vendors only need to build and distribute a single copy (for each OS / architecture), and users can install it on all Python versions from the [minimum version](#minimum-python-version-for-abi3) and up. The downside of this is that PyO3 can't use optimizations which rely on being compiled against a known exact Python version. It's up to you to decide whether this matters for your extension module. It's also possible to design your extension module such that you can distribute `abi3` wheels but allow users compiling from source to benefit from additional optimizations - see the [support for multiple python versions](./building-and-distribution/multiple-python-versions.md) section of this guide, in particular the `#[cfg(Py_LIMITED_API)]` flag.

There are three steps involved in making use of `abi3` when building Python packages as wheels:

1. Enable the `abi3` feature in `pyo3`. This ensures `pyo3` only calls Python C-API functions which are part of the stable API, and on Windows also ensures that the project links against the correct shared object (no special behavior is required on other platforms):

```toml
[dependencies]
pyo3 = { {{#PYO3_CRATE_VERSION}}, features = ["abi3"] }
```

2. Ensure that the built shared objects are correctly marked as `abi3`. This is accomplished by telling your build system that you're using the limited API. [`maturin`] >= 0.9.0 and [`setuptools-rust`] >= 0.11.4 support `abi3` wheels.
See the [corresponding](https://github.com/PyO3/maturin/pull/353) [PRs](https://github.com/PyO3/setuptools-rust/pull/82) for more.

3. Ensure that the `.whl` is correctly marked as `abi3`. For projects using `setuptools`, this is accomplished by passing `--py-limited-api=cp3x` (where `x` is the minimum Python version supported by the wheel, e.g. `--py-limited-api=cp35` for Python 3.5) to `setup.py bdist_wheel`.

#### Minimum Python version for `abi3`

Because a single `abi3` wheel can be used with many different Python versions, PyO3 has feature flags `abi3-py37`, `abi3-py38`, `abi3-py39` etc. to set the minimum required Python version for your `abi3` wheel.
For example, if you set the `abi3-py37` feature, your extension wheel can be used on all Python 3 versions from Python 3.7 and up. `maturin` and `setuptools-rust` will give the wheel a name like `my-extension-1.0-cp37-abi3-manylinux2020_x86_64.whl`.

As your extension module may be run with multiple different Python versions you may occasionally find you need to check the Python version at runtime to customize behavior. See [the relevant section of this guide](./building-and-distribution/multiple-python-versions.md#checking-the-python-version-at-runtime) on supporting multiple Python versions at runtime.

PyO3 is only able to link your extension module to abi3 version up to and including your host Python version. E.g., if you set `abi3-py38` and try to compile the crate with a host of Python 3.7, the build will fail.

> Note: If you set more that one of these `abi3` version feature flags the lowest version always wins. For example, with both `abi3-py37` and `abi3-py38` set, PyO3 would build a wheel which supports Python 3.7 and up.

#### Building `abi3` extensions without a Python interpreter

As an advanced feature, you can build PyO3 wheel without calling Python interpreter with the environment variable `PYO3_NO_PYTHON` set.
Also, if the build host Python interpreter is not found or is too old or otherwise unusable,
PyO3 will still attempt to compile `abi3` extension modules after displaying a warning message.
On Unix-like systems this works unconditionally; on Windows you must also set the `RUSTFLAGS` environment variable
to contain `-L native=/path/to/python/libs` so that the linker can find `python3.lib`.

If the `python3.dll` import library is not available, an experimental `generate-import-lib` crate
feature may be enabled, and the required library will be created and used by PyO3 automatically.

*Note*: MSVC targets require LLVM binutils (`llvm-dlltool`) to be available in `PATH` for
the automatic import library generation feature to work.

#### Missing features

Due to limitations in the Python API, there are a few `pyo3` features that do
not work when compiling for `abi3`. These are:

- `#[pyo3(text_signature = "...")]` does not work on classes until Python 3.10 or greater.
- The `dict` and `weakref` options on classes are not supported until Python 3.9 or greater.
- The buffer API is not supported until Python 3.11 or greater.
- Optimizations which rely on knowledge of the exact Python version compiled against.

## Embedding Python in Rust

If you want to embed the Python interpreter inside a Rust program, there are two modes in which this can be done: dynamically and statically. We'll cover each of these modes in the following sections. Each of them affect how you must distribute your program. Instead of learning how to do this yourself, you might want to consider using a project like [PyOxidizer] to ship your application and all of its dependencies in a single file.

PyO3 automatically switches between the two linking modes depending on whether the Python distribution you have configured PyO3 to use ([see above](#configuring-the-python-version)) contains a shared library or a static library. The static library is most often seen in Python distributions compiled from source without the `--enable-shared` configuration option.

### Dynamically embedding the Python interpreter

Embedding the Python interpreter dynamically is much easier than doing so statically. This is done by linking your program against a Python shared library (such as `libpython.3.9.so` on UNIX, or `python39.dll` on Windows). The implementation of the Python interpreter resides inside the shared library. This means that when the OS runs your Rust program it also needs to be able to find the Python shared library.

This mode of embedding works well for Rust tests which need access to the Python interpreter. It is also great for Rust software which is installed inside a Python virtualenv, because the virtualenv sets up appropriate environment variables to locate the correct Python shared library.

For distributing your program to non-technical users, you will have to consider including the Python shared library in your distribution as well as setting up wrapper scripts to set the right environment variables (such as `LD_LIBRARY_PATH` on UNIX, or `PATH` on Windows).

Note that PyPy cannot be embedded in Rust (or any other software). Support for this is tracked on the [PyPy issue tracker](https://github.com/pypy/pypy/issues/3836).

### Statically embedding the Python interpreter

Embedding the Python interpreter statically means including the contents of a Python static library directly inside your Rust binary. This means that to distribute your program you only need to ship your binary file: it contains the Python interpreter inside the binary!

On Windows static linking is almost never done, so Python distributions don't usually include a static library. The information below applies only to UNIX.

The Python static library is usually called `libpython.a`.

Static linking has a lot of complications, listed below. For these reasons PyO3 does not yet have first-class support for this embedding mode. See [issue 416 on PyO3's GitHub](https://github.com/PyO3/pyo3/issues/416) for more information and to discuss any issues you encounter.

The [`auto-initialize`](features.md#auto-initialize) feature is deliberately disabled when embedding the interpreter statically because this is often unintentionally done by new users to PyO3 running test programs. Trying out PyO3 is much easier using dynamic embedding.

The known complications are:
  - To import compiled extension modules (such as other Rust extension modules, or those written in C), your binary must have the correct linker flags set during compilation to export the original contents of `libpython.a` so that extensions can use them (e.g. `-Wl,--export-dynamic`).
  - The C compiler and flags which were used to create `libpython.a` must be compatible with your Rust compiler and flags, else you will experience compilation failures.

    Significantly different compiler versions may see errors like this:

    ```text
    lto1: fatal error: bytecode stream in file 'rust-numpy/target/release/deps/libpyo3-6a7fb2ed970dbf26.rlib' generated with LTO version 6.0 instead of the expected 6.2
    ```

    Mismatching flags may lead to errors like this:

    ```text
    /usr/bin/ld: /usr/lib/gcc/x86_64-linux-gnu/9/../../../x86_64-linux-gnu/libpython3.9.a(zlibmodule.o): relocation R_X86_64_32 against `.data' can not be used when making a PIE object; recompile with -fPIE
    ```

If you encounter these or other complications when linking the interpreter statically, discuss them on [issue 416 on PyO3's GitHub](https://github.com/PyO3/pyo3/issues/416). It is hoped that eventually that discussion will contain enough information and solutions that PyO3 can offer first-class support for static embedding.

### Import your module when embedding the Python interpreter

When you run your Rust binary with an embedded interpreter, any `#[pymodule]` created modules won't be accessible to import unless added to a table called `PyImport_Inittab` before the embedded interpreter is initialized. This will cause Python statements in your embedded interpreter such as `import your_new_module` to fail. You can call the macro [`append_to_inittab`]({{#PYO3_DOCS_URL}}/pyo3/macro.append_to_inittab.html) with your module before initializing the Python interpreter to add the module function into that table. (The Python interpreter will be initialized by calling `Python::initialize`, `with_embedded_python_interpreter`, or `Python::attach` with the [`auto-initialize`](features.md#auto-initialize) feature enabled.)

## Cross Compiling

Thanks to Rust's great cross-compilation support, cross-compiling using PyO3 is relatively straightforward. To get started, you'll need a few pieces of software:

* A toolchain for your target.
* The appropriate options in your Cargo `.config` for the platform you're targeting and the toolchain you are using.
* A Python interpreter that's already been compiled for your target (optional when building "abi3" extension modules).
* A Python interpreter that is built for your host and available through the `PATH` or setting the [`PYO3_PYTHON`](#configuring-the-python-version) variable (optional when building "abi3" extension modules).

After you've obtained the above, you can build a cross-compiled PyO3 module by using Cargo's `--target` flag. PyO3's build script will detect that you are attempting a cross-compile based on your host machine and the desired target.

When cross-compiling, PyO3's build script cannot execute the target Python interpreter to query the configuration, so there are a few additional environment variables you may need to set:

* `PYO3_CROSS`: If present this variable forces PyO3 to configure as a cross-compilation.
* `PYO3_CROSS_LIB_DIR`: This variable can be set to the directory containing the target's libpython DSO and the associated `_sysconfigdata*.py` file for Unix-like targets, or the Python DLL import libraries for the Windows target. This variable is only needed when the output binary must link to libpython explicitly (e.g. when targeting Windows and Android or embedding a Python interpreter), or when it is absolutely required to get the interpreter configuration from `_sysconfigdata*.py`.
* `PYO3_CROSS_PYTHON_VERSION`: Major and minor version (e.g. 3.9) of the target Python installation. This variable is only needed if PyO3 cannot determine the version to target from `abi3-py3*` features, or if `PYO3_CROSS_LIB_DIR` is not set, or if there are multiple versions of Python present in `PYO3_CROSS_LIB_DIR`.
* `PYO3_CROSS_PYTHON_IMPLEMENTATION`: Python implementation name ("CPython" or "PyPy") of the target Python installation. CPython is assumed by default when this variable is not set, unless `PYO3_CROSS_LIB_DIR` is set for a Unix-like target and PyO3 can get the interpreter configuration from `_sysconfigdata*.py`.

An experimental `pyo3` crate feature `generate-import-lib` enables the user to cross-compile
extension modules for Windows targets without setting the `PYO3_CROSS_LIB_DIR` environment
variable or providing any Windows Python library files. It uses an external [`python3-dll-a`] crate
to generate import libraries for the Python DLL for MinGW-w64 and MSVC compile targets.
`python3-dll-a` uses the binutils `dlltool` program to generate DLL import libraries for MinGW-w64 targets.
It is possible to override the default `dlltool` command name for the cross target
by setting `PYO3_MINGW_DLLTOOL` environment variable.
*Note*: MSVC targets require LLVM binutils or MSVC build tools to be available on the host system.
More specifically, `python3-dll-a` requires `llvm-dlltool` or `lib.exe` executable to be present in `PATH` when
targeting `*-pc-windows-msvc`. The Zig compiler executable can be used in place of `llvm-dlltool` when the `ZIG_COMMAND`
environment variable is set to the installed Zig program name (`"zig"` or `"python -m ziglang"`).

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

`PYO3_CROSS_LIB_DIR` can often be omitted when cross compiling extension modules for Unix and macOS targets,
or when cross compiling extension modules for Windows and the experimental `generate-import-lib`
crate feature is enabled.

The following resources may also be useful for cross-compiling:
 - [github.com/japaric/rust-cross](https://github.com/japaric/rust-cross) is a primer on cross compiling Rust.
 - [github.com/rust-embedded/cross](https://github.com/rust-embedded/cross) uses Docker to make Rust cross-compilation easier.

[`pyo3-build-config`]: https://github.com/PyO3/pyo3/tree/main/pyo3-build-config
[`maturin-starter`]: https://github.com/PyO3/pyo3/tree/main/examples/maturin-starter
[`setuptools-rust-starter`]: https://github.com/PyO3/pyo3/tree/main/examples/setuptools-rust-starter
[`maturin`]: https://github.com/PyO3/maturin
[`setuptools-rust`]: https://github.com/PyO3/setuptools-rust
[PyOxidizer]: https://github.com/indygreg/PyOxidizer
[`python3-dll-a`]: https://docs.rs/python3-dll-a/latest/python3_dll_a/
