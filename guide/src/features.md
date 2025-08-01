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

This feature changes [`Python::attach`]({{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.attach) to automatically initialize a Python interpreter (by calling [`Python::initialize`]({{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.initialize)) if needed.

If you do not enable this feature, you should call `Python::initialize()` before attempting to call any other Python APIs.

## Advanced Features

### `experimental-async`

This feature adds support for `async fn` in `#[pyfunction]` and `#[pymethods]`.

The feature has some unfinished refinements and performance improvements. To help finish this off, see [issue #1632](https://github.com/PyO3/pyo3/issues/1632) and its associated draft PRs.

### `experimental-inspect`

This feature adds to the built binaries introspection data that can be then retrieved using the `pyo3-introspection` crate to generate [type stubs](https://typing.readthedocs.io/en/latest/source/stubs.html).

Also, this feature adds the `pyo3::inspect` module, as well as `IntoPy::type_output` and `FromPyObject::type_input` APIs to produce Python type "annotations" for Rust types.

This is a first step towards adding first-class support for generating type annotations automatically in PyO3, however work is needed to finish this off. All feedback and offers of help welcome on [issue #2454](https://github.com/PyO3/pyo3/issues/2454).

### `py-clone`

This feature was introduced to ease migration. It was found that delayed reference counts cannot be made sound and hence `Clon`ing an instance of `Py<T>` must panic without the GIL being held. To avoid migrations introducing new panics without warning, the `Clone` implementation itself is now gated behind this feature.

### `pyo3_disable_reference_pool`

This is a performance-oriented conditional compilation flag, e.g. [set via `$RUSTFLAGS`][set-configuration-options], which disabled the global reference pool and the associated overhead for the crossing the Python-Rust boundary. However, if enabled, `Drop`ping an instance of `Py<T>` without the GIL being held will abort the process.

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

This feature enables each `#[pyclass]` to have more than one `#[pymethods]` block.

Most users should only need a single `#[pymethods]` per `#[pyclass]`. In addition, not all platforms (e.g. Wasm) are supported by `inventory`, which is used in the implementation of the feature. For this reason this feature is not enabled by default, meaning fewer dependencies and faster compilation for the majority of users.

See [the `#[pyclass]` implementation details](class.md#implementation-details) for more information.

### `nightly`

The `nightly` feature needs the nightly Rust compiler. This allows PyO3 to use the `auto_traits` and `negative_impls` features to fix the `Python::detach` function.

### `resolve-config`

The `resolve-config` feature of the `pyo3-build-config` crate controls whether that crate's
build script automatically resolves a Python interpreter / build configuration. This feature is primarily useful when building PyO3
itself. By default this feature is not enabled, meaning you can freely use `pyo3-build-config` as a standalone library to read or write PyO3 build configuration files or resolve metadata about a Python interpreter.

## Optional Dependencies

These features enable conversions between Python types and types from other Rust crates, enabling easy access to the rest of the Rust ecosystem.

### `anyhow`

Adds a dependency on [anyhow](https://docs.rs/anyhow). Enables a conversion from [anyhow](https://docs.rs/anyhow)’s [`Error`](https://docs.rs/anyhow/latest/anyhow/struct.Error.html) type to [`PyErr`]({{#PYO3_DOCS_URL}}/pyo3/struct.PyErr.html), for easy error handling.

### `arc_lock`

Enables Pyo3's `MutexExt` trait for all Mutexes that extend on [`lock_api::Mutex`](https://docs.rs/lock_api/latest/lock_api/struct.Mutex.html) or [`parking_lot::ReentrantMutex`](https://docs.rs/lock_api/latest/lock_api/struct.ReentrantMutex.html) and are wrapped in an [`Arc`](https://doc.rust-lang.org/std/sync/struct.Arc.html) type. Like [`Arc<parking_lot::Mutex>`](https://docs.rs/parking_lot/latest/parking_lot/type.Mutex.html#method.lock_arc)

### `bigdecimal`
Adds a dependency on [bigdecimal](https://docs.rs/bigdecimal) and enables conversions into its [`BigDecimal`](https://docs.rs/bigdecimal/latest/bigdecimal/struct.BigDecimal.html) type.

### `bytes`
Adds a dependency on [bytes](https://docs.rs/bytes/latest/bytes) and enables conversions into its [`Bytes`](https://docs.rs/bytes/latest/bytes/struct.Bytes.html) type.

### `chrono`

Adds a dependency on [chrono](https://docs.rs/chrono). Enables a conversion from [chrono](https://docs.rs/chrono)'s types to python:
- [TimeDelta](https://docs.rs/chrono/latest/chrono/struct.TimeDelta.html) -> [`PyDelta`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDelta.html)
- [FixedOffset](https://docs.rs/chrono/latest/chrono/offset/struct.FixedOffset.html) -> [`PyDelta`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDelta.html)
- [Utc](https://docs.rs/chrono/latest/chrono/offset/struct.Utc.html) -> [`PyTzInfo`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyTzInfo.html)
- [NaiveDate](https://docs.rs/chrono/latest/chrono/naive/struct.NaiveDate.html) -> [`PyDate`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDate.html)
- [NaiveTime](https://docs.rs/chrono/latest/chrono/naive/struct.NaiveTime.html) -> [`PyTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyTime.html)
- [DateTime](https://docs.rs/chrono/latest/chrono/struct.DateTime.html) -> [`PyDateTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDateTime.html)

### `chrono-local`

Enables conversion from and to [Local](https://docs.rs/chrono/latest/chrono/struct.Local.html) timezones.

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

### `jiff-02`

Adds a dependency on [jiff@0.2](https://docs.rs/jiff/0.2) and requires MSRV 1.70. Enables a conversion from [jiff](https://docs.rs/jiff)'s types to python:
- [SignedDuration](https://docs.rs/jiff/0.2/jiff/struct.SignedDuration.html) -> [`PyDelta`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDelta.html)
- [TimeZone](https://docs.rs/jiff/0.2/jiff/tz/struct.TimeZone.html) -> [`PyTzInfo`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyTzInfo.html)
- [Offset](https://docs.rs/jiff/0.2/jiff/tz/struct.Offset.html) -> [`PyTzInfo`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyTzInfo.html)
- [Date](https://docs.rs/jiff/0.2/jiff/civil/struct.Date.html) -> [`PyDate`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDate.html)
- [Time](https://docs.rs/jiff/0.2/jiff/civil/struct.Time.html) -> [`PyTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyTime.html)
- [DateTime](https://docs.rs/jiff/0.2/jiff/civil/struct.DateTime.html) -> [`PyDateTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDateTime.html)
- [Zoned](https://docs.rs/jiff/0.2/jiff/struct.Zoned.html) -> [`PyDateTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDateTime.html)
- [Timestamp](https://docs.rs/jiff/0.2/jiff/struct.Timestamp.html) -> [`PyDateTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDateTime.html)

### `lock_api`

Adds a dependency on [lock_api](https://docs.rs/lock_api) and enables Pyo3's `MutexExt` trait for all mutexes that extend on [`lock_api::Mutex`](https://docs.rs/lock_api/latest/lock_api/struct.Mutex.html) and [`parking_lot::ReentrantMutex`](https://docs.rs/lock_api/latest/lock_api/struct.ReentrantMutex.html) (like `parking_lot` or `spin`).

### `num-bigint`

Adds a dependency on [num-bigint](https://docs.rs/num-bigint) and enables conversions into its [`BigInt`](https://docs.rs/num-bigint/latest/num_bigint/struct.BigInt.html) and [`BigUint`](https://docs.rs/num-bigint/latest/num_bigint/struct.BigUint.html) types.

### `num-complex`

Adds a dependency on [num-complex](https://docs.rs/num-complex) and enables conversions into its [`Complex`](https://docs.rs/num-complex/latest/num_complex/struct.Complex.html) type.

### `num-rational`

Adds a dependency on [num-rational](https://docs.rs/num-rational) and enables conversions into its [`Ratio`](https://docs.rs/num-rational/latest/num_rational/struct.Ratio.html) type.

### `ordered-float`

Adds a dependency on [ordered-float](https://docs.rs/ordered-float) and enables conversions between [ordered-float](https://docs.rs/ordered-float)'s types and Python:
- [NotNan](https://docs.rs/ordered-float/latest/ordered_float/struct.NotNan.html) -> [`PyFloat`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyFloat.html)
- [OrderedFloat](https://docs.rs/ordered-float/latest/ordered_float/struct.OrderedFloat.html) -> [`PyFloat`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyFloat.html)

### `parking-lot`

Adds a dependency on [parking_lot](https://docs.rs/parking_lot) and enables Pyo3's `OnceExt` & `MutexExt` traits for [`parking_lot::Once`](https://docs.rs/parking_lot/latest/parking_lot/struct.Once.html) [`parking_lot::Mutex`](https://docs.rs/parking_lot/latest/parking_lot/type.Mutex.html) and [`parking_lot::ReentrantMutex`](https://docs.rs/parking_lot/latest/parking_lot/type.ReentrantMutex.html) types.

### `rust_decimal`

Adds a dependency on [rust_decimal](https://docs.rs/rust_decimal) and enables conversions into its [`Decimal`](https://docs.rs/rust_decimal/latest/rust_decimal/struct.Decimal.html) type.

### `time`

Adds a dependency on [time](https://docs.rs/time) and requires MSRV 1.67.1. Enables conversions between [time](https://docs.rs/time)'s types and Python:
- [Date](https://docs.rs/time/0.3.38/time/struct.Date.html) -> [`PyDate`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDate.html)
- [Time](https://docs.rs/time/0.3.38/time/struct.Time.html) -> [`PyTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyTime.html)
- [OffsetDateTime](https://docs.rs/time/0.3.38/time/struct.OffsetDateTime.html) -> [`PyDateTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDateTime.html)
- [PrimitiveDateTime](https://docs.rs/time/0.3.38/time/struct.PrimitiveDateTime.html) -> [`PyDateTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDateTime.html)
- [Duration](https://docs.rs/time/0.3.38/time/struct.Duration.html) -> [`PyDelta`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDelta.html)
- [UtcOffset](https://docs.rs/time/0.3.38/time/struct.UtcOffset.html) -> [`PyTzInfo`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyTzInfo.html)
- [UtcDateTime](https://docs.rs/time/0.3.38/time/struct.UtcDateTime.html) -> [`PyDateTime`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyDateTime.html)

### `serde`

Enables (de)serialization of `Py<T>` objects via [serde](https://serde.rs/).
This allows to use [`#[derive(Serialize, Deserialize)`](https://serde.rs/derive.html) on structs that hold references to `#[pyclass]` instances

```rust,no_run
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

### `uuid`

Adds a dependency on [uuid](https://docs.rs/uuid) and enables conversions into its [`Uuid`](https://docs.rs/uuid/latest/uuid/struct.Uuid.html) type.
