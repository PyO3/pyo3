# Features Reference

PyO3 provides a number of Cargo features to customise functionality. This chapter of the guide provides detail on each of them.

By default, the `macros` and `auto-initialize` features are enabled.

## Features for extension module authors

### `extension-module`

This feature is required when building a Python extension module using PyO3.

It tells PyO3's build script to skip linking against `libpython.so` on Unix platforms, where this must not be done.

See the [building and distribution](building_and_distribution.md#linking) section for further detail.

### `abi3`

This feature is used when building Python extension modules to create wheels which are compatible with multiple Python versions.

It restricts PyO3's API to a subset of the full Python API which is guaranteed by [PEP 384](https://www.python.org/dev/peps/pep-0384/) to be forwards-compatible with future Python versions.

See the [building and distribution](building_and_distribution.md#py_limited_apiabi3) section for further detail.

### `abi3-py36` / `abi3-py37` / `abi3-py38` / `abi3-py39`

These features are an extension of the `abi3` feature to specify the exact minimum Python version which the multiple-version-wheel will support.

See the [building and distribution](building_and_distribution.md#minimum-python-version-for-abi3) section for further detail.

## Features for embedding Python in Rust

### `auto-initalize`

This feature changes [`Python::with_gil`](https://docs.rs/pyo3/latest/pyo3/struct.Python.html#method.with_gil) and [`Python::acquire_gil`](https://docs.rs/pyo3/latest/pyo3/struct.Python.html#method.acquire_gil) to automatically initialize a Python interpreter (by calling [`prepare_freethreaded_python`](https://docs.rs/pyo3/latest/pyo3/fn.prepare_freethreaded_python.html)) if needed.

This feature is not needed for extension modules, but for compatibility it is enabled by default until at least the PyO3 0.14 release.

If you choose not to enable this feature, you should call `pyo3::prepare_freethreaded_python()` before attempting to call any other Python APIs.

> This feature is enabled by default. To disable it, set `default-features = false` for the `pyo3` entry in your Cargo.toml.

## Advanced Features

### `macros`

This feature enables a dependency on the `pyo3-macros` crate, which provides the procedural macros portion of PyO3's API:

- `#[pymodule]`
- `#[pyfunction]`
- `#[pyclass]`
- `#[pymethods]`
- `#[pyproto]`
- `#[derive(FromPyObject)]`

It also provides the `py_run!` macro.

These macros require a number of dependencies which may not be needed by users who just need PyO3 for Python FFI. Disabling this feature enables faster builds for those users, as these dependencies will not be built if this feature is disabled.

> This feature is enabled by default. To disable it, set `default-features = false` for the `pyo3` entry in your Cargo.toml.

### `nightly`

The `nightly` feature needs the nightly Rust compiler. This allows PyO3 to use Rust's unstable specialization feature to apply the following optimizations:
- `FromPyObject` for `Vec` and `[T;N]` can perform a `memcpy` when the object supports the Python buffer protocol.
- `ToBorrowedObject` can skip a reference count increase when the provided object is a Python native type.

### `serde`

The `serde` feature enables (de)serialization of Py<T> objects via [serde](https://serde.rs/).  
This allows to use [`#[derive(Serialize, Deserialize)`](https://serde.rs/derive.html) on structs that hold references to `#[pyclass]` instances

```rust

#[pyclass]
#[derive(Serialize, Deserialize)]
struct Permission {
    name: String
}

#[pyclass]
#[derive(Serialize, Deserialize)]
struct User {
    username: String,
    permissions: Vec<Py<Permission>>
}
```
