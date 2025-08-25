<!-- This file contains a rough overview of the PyO3 codebase. -->
<!-- Please do not make descriptions too specific, so that we can easily -->
<!-- keep this file in sync with the codebase. -->

# PyO3: Architecture

This document roughly describes the high-level architecture of PyO3.
If you want to become familiar with the codebase you are in the right place!

## Overview

PyO3 provides a bridge between Rust and Python, based on the [Python/C API].
Thus, PyO3 has low-level bindings of these API as its core.
On top of that, we have higher-level bindings to operate Python objects safely.
Also, to define Python classes and functions in Rust code, we have `trait PyClass` and a set of
protocol traits (e.g., `PyIterProtocol`) for supporting object protocols (i.e., `__dunder__` methods).
Since implementing `PyClass` requires lots of boilerplate, we have a proc-macro `#[pyclass]`.

To summarize, there are six main parts to the PyO3 codebase.

1. [Low-level bindings of Python/C API.](#1-low-level-bindings-of-python-capi)
   - [`pyo3-ffi`] and [`src/ffi`]
2. [Bindings to Python objects.](#2-bindings-to-python-objects)
   - [`src/instance.rs`] and [`src/types`]
3. [`PyClass` and related functionalities.](#3-pyclass-and-related-functionalities)
   - [`src/pycell.rs`], [`src/pyclass.rs`], and more
4. [Procedural macros to simplify usage for users.](#4-procedural-macros-to-simplify-usage-for-users)
   - [`src/impl_`], [`pyo3-macros`] and [`pyo3-macros-backend`]
5. [`build.rs` and `pyo3-build-config`](#5-buildrs-and-pyo3-build-config)
   - [`build.rs`](https://github.com/PyO3/pyo3/tree/main/build.rs)
   - [`pyo3-build-config`]

## 1. Low-level bindings of Python/C API

[`pyo3-ffi`] contains wrappers of the [Python/C API]. This is currently done by hand rather than
automated tooling because:
  - it gives us best control about how to adapt C conventions to Rust, and
  - there are many Python interpreter versions we support in a single set of files.

We aim to provide straight-forward Rust wrappers resembling the file structure of [`cpython/Include`](https://github.com/python/cpython/tree/main/Include).

We are continuously updating the module to match the latest CPython version which PyO3 supports (i.e. as of time of writing Python 3.13). The tracking issue is [#1289](https://github.com/PyO3/pyo3/issues/1289), and contribution is welcome.

In the [`pyo3-ffi`] crate, there is lots of conditional compilation such as `#[cfg(Py_LIMITED_API)]`,
`#[cfg(Py_3_7)]`, and `#[cfg(PyPy)]`.
`Py_LIMITED_API` corresponds to `#define Py_LIMITED_API` macro in Python/C API.
With `Py_LIMITED_API`, we can build a Python-version-agnostic binary called an
[abi3 wheel](https://pyo3.rs/latest/building-and-distribution.html#py_limited_apiabi3).
`Py_3_7` means that the API is available from Python >= 3.7.
There are also `Py_3_8`, `Py_3_9`, and so on.
`PyPy` means that the API definition is for PyPy.
Those flags are set in [`build.rs`](#6-buildrs-and-pyo3-build-config).

## 2. Bindings to Python objects

[`src/types`] contains bindings to [built-in types](https://docs.python.org/3/library/stdtypes.html)
of Python, such as `dict` and `list`.
For historical reasons, Python's `object` is called `PyAny` in PyO3 and located in [`src/types/any.rs`].

Currently, `PyAny` is a straightforward wrapper of `ffi::PyObject`, defined as:

```rust
#[repr(transparent)]
pub struct PyAny(UnsafeCell<ffi::PyObject>);
```

Concrete Python objects are implemented by wrapping `PyAny`, e.g.,:

```rust
#[repr(transparent)]
pub struct PyDict(PyAny);
```

These types are not intended to be accessed directly, and instead are used through the `Py<T>` and `Bound<T>` smart pointers.

We have some macros in [`src/types/mod.rs`] which make it easier to implement APIs for concrete Python types.

## 3. `PyClass` and related functionalities

[`src/pycell.rs`], [`src/pyclass.rs`], and [`src/type_object.rs`] contain types and
traits to make `#[pyclass]` work.
Also, [`src/pyclass_init.rs`] and [`src/impl_/pyclass.rs`] have related functionalities.

To realize object-oriented programming in C, all Python objects have `ob_base: PyObject` as their
first field in their structure definition. Thanks to this guarantee, casting `*mut A` to `*mut PyObject`
is valid if `A` is a Python object.

To ensure this guarantee, we have a wrapper struct `PyClassObject<T>` in [`src/pycell/impl_.rs`] which is roughly:

```rust
#[repr(C)]
pub struct PyClassObject<T> {
    ob_base: crate::ffi::PyObject,
    inner: T,
}
```

Thus, when copying a Rust struct to a Python object, we first allocate `PyClassObject` on the Python heap and then
move `T` into it.

The primary way to interact with Python objects implemented in Rust is through the `Bound<'py, T>` smart pointer.
By having the `'py` lifetime of the `Python<'py>` token, this ties the lifetime of the `Bound<'py, T>` smart pointer to the lifetime of the GIL and allows PyO3 to call Python APIs at maximum efficiency.

`Bound<'py, T>` requires that `T` implements `PyClass`.
This trait is somewhat complex and derives many traits, but the most important one is `PyTypeInfo`
in [`src/type_object.rs`].
`PyTypeInfo` is also implemented for built-in types.
In Python, all objects have their types, and types are also objects of `type`.
For example, you can see `type({})` shows `dict` and `type(type({}))` shows `type` in Python REPL.
`T: PyTypeInfo` implies that `T` has a corresponding type object.

### Protocol methods

Python has some built-in special methods called dunder methods, such as `__iter__`.
They are called "slots" in the [abstract objects layer](https://docs.python.org/3/c-api/abstract.html) in
Python/C API.
We provide a way to implement those protocols similarly, by recognizing special
names in `#[pymethods]`, with a few new ones for slots that can not be
implemented in Python, such as GC support.

## 4. Procedural macros to simplify usage for users.

[`pyo3-macros`] provides five proc-macro APIs: `pymodule`, `pyfunction`, `pyclass`,
`pymethods`, and `#[derive(FromPyObject)]`.
[`pyo3-macros-backend`] has the actual implementations of these APIs.
[`src/impl_`] contains `#[doc(hidden)]` functionality used in code generated by these proc-macros,
such as parsing function arguments.

## 5. `build.rs` and `pyo3-build-config`

PyO3 supports a wide range of OSes, interpreters and use cases. The correct environment must be
detected at build time in order to set up relevant conditional compilation correctly. This logic
is captured in the [`pyo3-build-config`] crate, which is a `build-dependency` of `pyo3` and
`pyo3-macros`, and can also be used by downstream users in the same way.

In [`pyo3-build-config`]'s `build.rs` the build environment is detected and inlined into the crate
as a "config file". This works in all cases except for cross-compiling, where it is necessary to
capture this from the `pyo3` `build.rs` to get some extra environment variables that Cargo doesn't
set for build dependencies.

The `pyo3` `build.rs` also runs some safety checks such as ensuring the Python version detected is
actually supported.

Some of the functionality of `pyo3-build-config`:
- Find the interpreter for build and detect the Python version.
  - We have to set some version flags like `#[cfg(Py_3_7)]`.
  - If the interpreter is PyPy, we set `#[cfg(PyPy)`.
  - If the `PYO3_CONFIG_FILE` environment variable is set then that file's contents will be used
    instead of any detected configuration.
  - If the `PYO3_NO_PYTHON` environment variable is set then the interpreter detection is bypassed
    entirely and only abi3 extensions can be built.
- Check if we are building a Python extension.
  - If we are building an extension (e.g., Python library installable by `pip`),
    we don't link `libpython`.
    Currently we use the `extension-module` feature for this purpose. This may change in the future.
    See [#1123](https://github.com/PyO3/pyo3/pull/1123).
- Cross-compiling configuration
  - If `TARGET` architecture and `HOST` architecture differ, we can find cross compile information
    from environment variables (`PYO3_CROSS_LIB_DIR`, `PYO3_CROSS_PYTHON_VERSION` and
    `PYO3_CROSS_PYTHON_IMPLEMENTATION`) or system files.
    When cross compiling extension modules it is often possible to make it work without any
    additional user input.
  - When an experimental feature `generate-import-lib` is enabled, the `pyo3-ffi` build script can
    generate `python3.dll` import libraries for Windows targets automatically via an external
    [`python3-dll-a`] crate. This enables the users to cross compile Python extensions for Windows without
    having to install any Windows Python libraries.

<!-- External Links -->

[python/c api]: https://docs.python.org/3/c-api/
[`python3-dll-a`]: https://docs.rs/python3-dll-a/latest/python3_dll_a/

<!-- Crates -->

[`pyo3-macros`]: https://github.com/PyO3/pyo3/tree/main/pyo3-macros
[`pyo3-macros-backend`]: https://github.com/PyO3/pyo3/tree/main/pyo3-macros-backend
[`pyo3-build-config`]: https://github.com/PyO3/pyo3/tree/main/pyo3-build-config
[`pyo3-ffi`]: https://github.com/PyO3/pyo3/tree/main/pyo3-ffi

<!-- Directories -->

[`src/class`]: https://github.com/PyO3/pyo3/tree/main/src/class
[`src/ffi`]: https://github.com/PyO3/pyo3/tree/main/src/ffi
[`src/types`]: https://github.com/PyO3/pyo3/tree/main/src/types

<!-- Files -->

[`src/impl_`]: https://github.com/PyO3/pyo3/blob/main/src/impl_
[`src/instance.rs`]: https://github.com/PyO3/pyo3/tree/main/src/instance.rs
[`src/pycell.rs`]: https://github.com/PyO3/pyo3/tree/main/src/pycell.rs
[`src/pyclass.rs`]: https://github.com/PyO3/pyo3/tree/main/src/pyclass.rs
[`src/pyclass_init.rs`]: https://github.com/PyO3/pyo3/tree/main/src/pyclass_init.rs
[`src/pyclass_slot.rs`]: https://github.com/PyO3/pyo3/tree/main/src/pyclass_slot.rs
[`src/type_object.rs`]: https://github.com/PyO3/pyo3/tree/main/src/type_object.rs
[`src/class/methods.rs`]: https://github.com/PyO3/pyo3/tree/main/src/class/methods.rs
[`src/class/impl_.rs`]: https://github.com/PyO3/pyo3/tree/main/src/class/impl_.rs
[`src/types/any.rs`]: https://github.com/PyO3/pyo3/tree/main/src/types/any.rs
[`src/types/mod.rs`]: https://github.com/PyO3/pyo3/tree/main/src/types/mod.rs
