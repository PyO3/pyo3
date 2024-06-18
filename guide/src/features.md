# Features reference

PyO3 provides a number of Cargo features to customize functionality. This chapter of the guide provides detail on each of them.

By default, only the `macros` feature is enabled.

## Features for extension module authors

### `extension-module`

This feature is required when building a Python extension module using PyO3.

It tells PyO3's build script to skip linking against `libpython.so` on Unix platforms, where this must not be done.

See the [building and distribution](building-and-distribution.md#the-extension-module-feature) section for further detail.

### `abi3`

This feature is used when building Python extension modules to create wheels which are compatible with multiple Python versions.

It restricts PyO3's API to a subset of the full Python API which is guaranteed by [PEP 384](https://www.python.org/dev/peps/pep-0384/) to be forwards-compatible with future Python versions.

See the [building and distribution](building-and-distribution.md#py_limited_apiabi3) section for further detail.

### The `abi3-pyXY` features

(`abi3-py37`, `abi3-py38`, `abi3-py39`, `abi3-py310` and `abi3-py311`)

These features are extensions of the `abi3` feature to specify the exact minimum Python version which the multiple-version-wheel will support.

See the [building and distribution](building-and-distribution.md#minimum-python-version-for-abi3) section for further detail.

### `generate-import-lib`

This experimental feature is used to generate import libraries for Python DLL
for MinGW-w64 and MSVC (cross-)compile targets.

Enabling it allows to (cross-)compile extension modules to any Windows targets
without having to install the Windows Python distribution files for the target.

See the [building and distribution](building-and-distribution.md#building-abi3-extensions-without-a-python-interpreter)
section for further detail.

## Features for embedding Python in Rust

### `auto-initialize`

This feature changes [`Python::with_gil`]({{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.with_gil) to automatically initialize a Python interpreter (by calling [`prepare_freethreaded_python`]({{#PYO3_DOCS_URL}}/pyo3/fn.prepare_freethreaded_python.html)) if needed.

If you do not enable this feature, you should call `pyo3::prepare_freethreaded_python()` before attempting to call any other Python APIs.

## Advanced Features

### `experimental-async`

This feature adds support for `async fn` in `#[pyfunction]` and `#[pymethods]`.

The feature has some unfinished refinements and performance improvements. To help finish this off, see [issue #1632](https://github.com/PyO3/pyo3/issues/1632) and its associated draft PRs.

### `experimental-inspect`

This feature adds the `pyo3::inspect` module, as well as `IntoPy::type_output` and `FromPyObject::type_input` APIs to produce Python type "annotations" for Rust types.

This is a first step towards adding first-class support for generating type annotations automatically in PyO3, however work is needed to finish this off. All feedback and offers of help welcome on [issue #2454](https://github.com/PyO3/pyo3/issues/2454).

### `gil-refs`

This feature is a backwards-compatibility feature to allow continued use of the "GIL Refs" APIs deprecated in PyO3 0.21. These APIs have performance drawbacks and soundness edge cases which the newer `Bound<T>` smart pointer and accompanying APIs resolve.

This feature and the APIs it enables is expected to be removed in a future PyO3 version.

### `py-clone`

This feature was introduced to ease migration. It was found that delayed reference counts cannot be made sound and hence `Clon`ing an instance of `Py<T>` must panic without the GIL being held. To avoid migrations introducing new panics without warning, the `Clone` implementation itself is now gated behind this feature.

### `pyo3_disable_reference_pool`

This is a performance-oriented conditional compilation flag, e.g. [set via `$RUSTFLAGS`][set-configuration-options], which disabled the global reference pool and the assocaited overhead for the crossing the Python-Rust boundary. However, if enabled, `Drop`ping an instance of `Py<T>` without the GIL being held will abort the process.

### `macros`

This feature enables a dependency on the `pyo3-macros` crate, which provides the procedural macros portion of PyO3's API:

- `#[pymodule]`
- `#[pyfunction]`
- `#[pyclass]`
- `#[pymethods]`
- `#[derive(FromPyObject)]`

It also provides the `py_run!` macro.

These macros require a number of dependencies which may not be needed by users who just need PyO3 for Python FFI. Disabling this feature enables faster builds for those users, as these dependencies will not be built if this feature is disabled.

> This feature is enabled by default. To disable it, set `default-features = false` for the `pyo3` entry in your Cargo.toml.

### `multiple-pymethods`

This feature enables a dependency on `inventory`, which enables each `#[pyclass]` to have more than one `#[pymethods]` block. This feature also requires a minimum Rust version of 1.62 due to limitations in the `inventory` crate.

Most users should only need a single `#[pymethods]` per `#[pyclass]`. In addition, not all platforms (e.g. Wasm) are supported by `inventory`. For this reason this feature is not enabled by default, meaning fewer dependencies and faster compilation for the majority of users.

See [the `#[pyclass]` implementation details](class.md#implementation-details) for more information.

### `nightly`

The `nightly` feature needs the nightly Rust compiler. This allows PyO3 to use the `auto_traits` and `negative_impls` features to fix the `Python::allow_threads` function.

### `resolve-config`

The `resolve-config` feature of the `pyo3-build-config` crate controls whether that crate's
build script automatically resolves a Python interpreter / build configuration. This feature is primarily useful when building PyO3
itself. By default this feature is not enabled, meaning you can freely use `pyo3-build-config` as a standalone library to read or write PyO3 build configuration files or resolve metadata about a Python interpreter.

## Optional Dependencies

These features enable conversions between Python types and types from other Rust crates, enabling easy access to the rest of the Rust ecosystem.

### `anyhow`

Adds a dependency on [anyhow](https://docs.rs/anyhow). Enables a conversion from [anyhow](https://docs.rs/anyhow)’s [`Error`](https://docs.rs/anyhow/latest/anyhow/struct.Error.html) type to [`PyErr`]({{#PYO3_DOCS_URL}}/pyo3/struct.PyErr.html), for easy error handling.

### `chrono`

Adds a dependency on [chrono](https://docs.rs/chrono). Enables a conversion from [chrono](https://docs.rs/chrono)'s types to python:
- [TimeDelta](https://docs.rs/chrono/latest/chrono/struct.TimeDelta.html) -> [`PyDelta`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDelta.html)
- [FixedOffset](https://docs.rs/chrono/latest/chrono/offset/struct.FixedOffset.html) -> [`PyDelta`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDelta.html)
- [Utc](https://docs.rs/chrono/latest/chrono/offset/struct.Utc.html) -> [`PyTzInfo`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyTzInfo.html)
- [NaiveDate](https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDate.html) -> [`PyDate`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDate.html)
- [NaiveTime](https://docs.rs/chrono/latest/chrono/naive/struct.NaiveTime.html) -> [`PyTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyTime.html)
- [DateTime](https://docs.rs/chrono/latest/chrono/struct.DateTime.html) -> [`PyDateTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDateTime.html)

### `chrono-tz`

Adds a dependency on [chrono-tz](https://docs.rs/chrono-tz).
Enables conversion from and to [`Tz`](https://docs.rs/chrono-tz/latest/chrono_tz/enum.Tz.html).
It requires at least Python 3.9.

### `either`

Adds a dependency on [either](https://docs.rs/either). Enables a conversions into [either](https://docs.rs/either)’s [`Either`](https://docs.rs/either/latest/either/enum.Either.html) type.

### `eyre`

Adds a dependency on [eyre](https://docs.rs/eyre). Enables a conversion from [eyre](https://docs.rs/eyre)’s [`Report`](https://docs.rs/eyre/latest/eyre/struct.Report.html) type to [`PyErr`]({{#PYO3_DOCS_URL}}/pyo3/struct.PyErr.html), for easy error handling.

### `hashbrown`

Adds a dependency on [hashbrown](https://docs.rs/hashbrown) and enables conversions into its [`HashMap`](https://docs.rs/hashbrown/latest/hashbrown/struct.HashMap.html) and [`HashSet`](https://docs.rs/hashbrown/latest/hashbrown/struct.HashSet.html) types.

### `indexmap`

Adds a dependency on [indexmap](https://docs.rs/indexmap) and enables conversions into its [`IndexMap`](https://docs.rs/indexmap/latest/indexmap/map/struct.IndexMap.html) type.

### `num-bigint`

Adds a dependency on [num-bigint](https://docs.rs/num-bigint) and enables conversions into its [`BigInt`](https://docs.rs/num-bigint/latest/num_bigint/struct.BigInt.html) and [`BigUint`](https://docs.rs/num-bigint/latest/num_bigint/struct.BigUint.html) types.

### `num-complex`

Adds a dependency on [num-complex](https://docs.rs/num-complex) and enables conversions into its [`Complex`](https://docs.rs/num-complex/latest/num_complex/struct.Complex.html) type.

### `num-rational`

Adds a dependency on [num-rational](https://docs.rs/num-rational) and enables conversions into its [`Ratio`](https://docs.rs/num-rational/latest/num_rational/struct.Ratio.html) type.

### `rust_decimal`

Adds a dependency on [rust_decimal](https://docs.rs/rust_decimal) and enables conversions into its [`Decimal`](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html) type.

### `serde`

Enables (de)serialization of `Py<T>` objects via [serde](https://serde.rs/).
This allows to use [`#[derive(Serialize, Deserialize)`](https://serde.rs/derive.html) on structs that hold references to `#[pyclass]` instances

```rust
# #[cfg(feature = "serde")]
# #[allow(dead_code)]
# mod serde_only {
# use pyo3::prelude::*;
# use serde::{Deserialize, Serialize};

#[pyclass]
#[derive(Serialize, Deserialize)]
struct Permission {
    name: String,
}

#[pyclass]
#[derive(Serialize, Deserialize)]
struct User {
    username: String,
    permissions: Vec<Py<Permission>>,
}
# }
```

### `smallvec`

Adds a dependency on [smallvec](https://docs.rs/smallvec) and enables conversions into its [`SmallVec`](https://docs.rs/smallvec/latest/smallvec/struct.SmallVec.html) type.

[set-configuration-options]: https://doc.rust-lang.org/reference/conditional-compilation.html#set-configuration-options
