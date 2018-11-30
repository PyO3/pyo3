# Building and Distribution

## Python version

pyo3 uses a build script to determine the python version and set the correct linker arguments. By default it uses the `python3` executable. With the `python2` feature it uses the `python2` executable. You can override the python interpreter by setting `PYTHON_SYS_EXECUTABLE`. Note that you still need to activate the `python2` with  `PYTHON_SYS_EXECUTABLE=python2` (see [pyo3/pyo3#276](https://github.com/PyO3/pyo3/issues/276) for details).

## Linking

Different linker arguments must be set for libraries/extension modules and binaries, which includes both standalone binaries and tests. (More specifically, binaries must be told where to find libpython and libraries must not link to libpython for [manylinux](https://www.python.org/dev/peps/pep-0513/) compliance).

Since pyo3's build script can't know whether you're building a binary or a library, you have to activate the `extension-module` feature to get the build options for a library, or it'll default to binary.

If you have e.g. a library crate and a profiling crate alongside, you need to use optional features. E.g. you put the following in the library crate:

```toml
[dependencies]
pyo3 = "0.5.1"

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
pyo3 = "0.5.2"
```

On linux/mac you might have to change `LD_LIBRARY_PATH` to include libpython, while on windows you might need to set `LIB` to include `pythonxy.lib` (where x and y are major and minor version), which is normally either in the `libs` or `Lib` folder of a python installation.

## Distribution

There are two ways to distribute your module as python package: The old [setuptools-rust](https://github.com/PyO3/setuptools-rust) and the new [pyo3-pack](https://github.com/pyo3/pyo3-pack). setuptools-rust needs some configuration files (`setup.py`,  `MANIFEST.in`, `build-wheels.sh`, etc.) and external tools (docker, twine). pyo3-pack doesn't need any configuration files. It can not yet build sdist though ([pyo3/pyo3-pack#2](https://github.com/PyO3/pyo3-pack/issues/2)).
