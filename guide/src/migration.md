# Migrating from older PyO3 versions

This guide can help you upgrade code through breaking changes from one PyO3 version to the next.
For a detailed list of all changes, see the [CHANGELOG](changelog.md).

## from 0.25.* to 0.26
### Rename of `Python::with_gil`, `Python::allow_threads`, and `pyo3::prepare_freethreaded_python`
<details open>
<summary><small>Click to expand</small></summary>

The names for these APIs were created when the global interpreter lock (GIL) was mandatory. With the introduction of free-threading in Python 3.13 this is no longer the case, and the naming does not has no universal meaning anymore.
For this reason we chose to rename these to more modern terminology introduced in free-threading:

- `Python::with_gil` is now called `Python::attach`, it attaches a Python thread-state to the current thread. In GIL enabled builds there can only be 1 thread attached to the interpreter, in free-threading there can be more.
- `Python::allow_threads` is now called `Python::detach`, it detaches a previously attached thread-state.
- `pyo3::prepare_freethreaded_python` is now called `Python::initialize`.
</details>

### Deprecation of `PyObject` type alias
<details open>
<summary><small>Click to expand</small></summary>

The type alias `PyObject` (aka `Py<PyAny>`) is often confused with the identically named FFI definition `pyo3::ffi::PyObject`. For this reason we are deprecating its usage. To migrate simply replace its usage by the target type `Py<PyAny>`.
</details>

### Replacement of `GILOnceCell` with `PyOnceLock`
<details open>
<summary><small>Click to expand</small></summary>

Similar to the above renaming of `Python::with_gil` and related APIs, the `GILOnceCell` type was designed for a Python interpreter which was limited by the GIL. Aside from its name, it allowed for the "once" initialization to race because the racing was mediated by the GIL and was extremely unlikely to manifest in practice.

With the introduction of free-threaded Python the racy initialization behavior is more likely to be problematic and so a new type `PyOnceLock` has been introduced which performs true single-initialization correctly while attached to the Python interpreter. It exposes the same API as `GILOnceCell`, so should be a drop-in replacement with the notable exception that if the racy initialization of `GILOnceCell` was inadvertently relied on (e.g. due to circular references) then the stronger once-ever guarantee of `PyOnceLock` may lead to deadlocking which requires refactoring.

Before:

```rust
# #![allow(deprecated)]
# use pyo3::prelude::*;
# use pyo3::sync::GILOnceCell;
# use pyo3::types::PyType;
# fn main() -> PyResult<()> {
# Python::attach(|py| {
static DECIMAL_TYPE: GILOnceCell<Py<PyType>> = GILOnceCell::new();
DECIMAL_TYPE.import(py, "decimal", "Decimal")?;
# Ok(())
# })
# }
```

After:

```rust
# use pyo3::prelude::*;
# use pyo3::sync::PyOnceLock;
# use pyo3::types::PyType;
# fn main() -> PyResult<()> {
# Python::attach(|py| {
static DECIMAL_TYPE: PyOnceLock<Py<PyType>> = PyOnceLock::new();
DECIMAL_TYPE.import(py, "decimal", "Decimal")?;
# Ok(())
# })
# }
```
</details>

### Deprecation of `GILProtected`
<details open>
<summary><small>Click to expand</small></summary>

As another cleanup related to concurrency primitives designed for a Python constrained by the GIL, the `GILProtected` type is now deprecated. Prefer to use concurrency primitives which are compatible with free-threaded Python, such as [`std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html) (in combination with PyO3's [`MutexExt`]({{#PYO3_DOCS_URL}}/pyo3/sync/trait.MutexExt.html) trait).

Before:

```rust
# #![allow(deprecated)]
# use pyo3::prelude::*;
# fn main() {
# #[cfg(not(Py_GIL_DISABLED))] {
use pyo3::sync::GILProtected;
use std::cell::RefCell;
# Python::attach(|py| {
static NUMBERS: GILProtected<RefCell<Vec<i32>>> = GILProtected::new(RefCell::new(Vec::new()));
Python::attach(|py| {
    NUMBERS.get(py).borrow_mut().push(42);
});
# })
# }
# }
```

After:

```rust
# use pyo3::prelude::*;
use pyo3::sync::MutexExt;
use std::sync::Mutex;
# fn main() {
# Python::attach(|py| {
static NUMBERS: Mutex<Vec<i32>> = Mutex::new(Vec::new());
Python::attach(|py| {
    NUMBERS.lock_py_attached(py).expect("no poisoning").push(42);
});
# })
# }
```
</details>

### `PyMemoryError` now maps to `io::ErrorKind::OutOfMemory` when converted to `io::Error`
<details>
<summary><small>Click to expand</small></summary>

Previously, converting a `PyMemoryError` into a Rust `io::Error` would result in an error with kind `Other`. Now, it produces an error with kind `OutOfMemory`.
Similarly, converting an `io::Error` with kind `OutOfMemory` back into a Python error would previously yield a generic `PyOSError`. Now, it yields a `PyMemoryError`.

This change makes error conversions more precise and matches the semantics of out-of-memory errors between Python and Rust.
</details>

## from 0.24.* to 0.25
### `AsPyPointer` removal
<details>
<summary><small>Click to expand</small></summary>
The `AsPyPointer` trait is mostly a leftover from the now removed gil-refs API. The last remaining uses were the GC API, namely `PyVisit::call`, and identity comparison (`PyAnyMethods::is` and `Py::is`).

`PyVisit::call` has been updated to take `T: Into<Option<&Py<T>>>`, which allows for arguments of type `&Py<T>`, `&Option<Py<T>>` and `Option<&Py<T>>`. It is unlikely any changes are needed here to migrate.

`PyAnyMethods::is`/`Py::is` has been updated to take `T: AsRef<Py<PyAny>>>`. Additionally `AsRef<Py<PyAny>>>` implementations were added for `Py`, `Bound` and `Borrowed`. Because of the existing `AsRef<Bound<PyAny>> for Bound<T>` implementation this may cause inference issues in non-generic code. This can be easily migrated by switching to `as_any` instead of `as_ref` for these calls.
</details>

## from 0.23.* to 0.24
<details>
<summary><small>Click to expand</small></summary>
There were no significant changes from 0.23 to 0.24 which required documenting in this guide.
</details>

## from 0.22.* to 0.23
<details>
<summary><small>Click to expand</small></summary>

PyO3 0.23 is a significant rework of PyO3's internals for two major improvements:
 - Support of Python 3.13's new freethreaded build (aka "3.13t")
 - Rework of to-Python conversions with a new `IntoPyObject` trait.

These changes are both substantial and reasonable efforts have been made to allow as much code as possible to continue to work as-is despite the changes. The impacts are likely to be seen in three places when upgrading:
 - PyO3's data structures [are now thread-safe](#free-threaded-python-support) instead of reliant on the GIL for synchronization. In particular, `#[pyclass]` types are [now required to be `Sync`](./class/thread-safety.md).
 - The [`IntoPyObject` trait](#new-intopyobject-trait-unifies-to-python-conversions) may need to be implemented for types in your codebase. In most cases this can simply be done with [`#[derive(IntoPyObject)]`](#intopyobject-and-intopyobjectref-derive-macros). There will be many deprecation warnings from the replacement of `IntoPy` and `ToPyObject` traits.
 - There will be many deprecation warnings from the [final removal of the `gil-refs` feature](#gil-refs-feature-removed), which opened up API space for a cleanup and simplification to PyO3's "Bound" API.

The sections below discuss the rationale and details of each change in more depth.
</details>

### Free-threaded Python Support
<details>
<summary><small>Click to expand</small></summary>

PyO3 0.23 introduces initial support for the new free-threaded build of
CPython 3.13, aka "3.13t".

Because this build allows multiple Python threads to operate simultaneously on underlying Rust data, the `#[pyclass]` macro now requires that types it operates on implement `Sync`.

Aside from the change to `#[pyclass]`, most features of PyO3 work unchanged, as the changes have been to the internal data structures to make them thread-safe. An example of this is the `GILOnceCell` type, which used the GIL to synchronize single-initialization. It now uses internal locks to guarantee that only one write ever succeeds, however it allows for multiple racing runs of the initialization closure. It may be preferable to instead use `std::sync::OnceLock` in combination with the `pyo3::sync::OnceLockExt` trait which adds `OnceLock::get_or_init_py_attached` for single-initialization where the initialization closure is guaranteed only ever to run once and without deadlocking with the GIL.

Future PyO3 versions will likely add more traits and data structures to make working with free-threaded Python easier.

Some features are unaccessible on the free-threaded build:
  - The `GILProtected` type, which relied on the GIL to expose synchronized access to inner contents
  - `PyList::get_item_unchecked`, which cannot soundly be used due to races between time-of-check and time-of-use

If you make use of these features then you will need to account for the
unavailability of the API in the free-threaded build. One way to handle it is via conditional compilation -- extensions can use `pyo3-build-config` to get access to a `#[cfg(Py_GIL_DISABLED)]` guard.

See [the guide section on free-threaded Python](free-threading.md) for more details about supporting free-threaded Python in your PyO3 extensions.
</details>

### New `IntoPyObject` trait unifies to-Python conversions
<details>
<summary><small>Click to expand</small></summary>

PyO3 0.23 introduces a new `IntoPyObject` trait to convert Rust types into Python objects which replaces both `IntoPy` and `ToPyObject`.
Notable features of this new trait include:
- conversions can now return an error
- it is designed to work efficiently for both `T` owned types and `&T` references
- compared to `IntoPy<T>` the generic `T` moved into an associated type, so
  - there is now only one way to convert a given type
  - the output type is stronger typed and may return any Python type instead of just `PyAny`
- byte collections are specialized to convert into `PyBytes` now, see [below](#to-python-conversions-changed-for-byte-collections-vecu8-u8-n-and-smallvecu8-n)
- `()` (unit) is now only specialized in return position of `#[pyfunction]` and `#[pymethods]` to return `None`, in normal usage it converts into an empty `PyTuple`

All PyO3 provided types as well as `#[pyclass]`es already implement `IntoPyObject`. Other types will
need to adapt an implementation of `IntoPyObject` to stay compatible with the Python APIs. In many cases
the new [`#[derive(IntoPyObject)]`](#intopyobject-and-intopyobjectref-derive-macros) macro can be used instead of
[manual implementations](#intopyobject-manual-implementation).

Since `IntoPyObject::into_pyobject` may return either a `Bound` or `Borrowed`, you may find the [`BoundObject`](conversions/traits.md#boundobject-for-conversions-that-may-be-bound-or-borrowed) trait to be useful to write code that generically handles either type of smart pointer.

Together with the introduction of `IntoPyObject` the old conversion traits `ToPyObject` and `IntoPy`
are deprecated and will be removed in a future PyO3 version.

#### `IntoPyObject` and `IntoPyObjectRef` derive macros

To implement the new trait you may use the new `IntoPyObject` and `IntoPyObjectRef` derive macros as below.

```rust,no_run
# use pyo3::prelude::*;
#[derive(IntoPyObject, IntoPyObjectRef)]
struct Struct {
    count: usize,
    obj: Py<PyAny>,
}
```

The `IntoPyObjectRef` derive macro derives implementations for references (e.g. for `&Struct` in the example above), which is a replacement for the `ToPyObject` trait.

#### `IntoPyObject` manual implementation

Before:
```rust,ignore
# use pyo3::prelude::*;
# #[allow(dead_code)]
struct MyPyObjectWrapper(PyObject);

impl IntoPy<PyObject> for MyPyObjectWrapper {
    fn into_py(self, py: Python<'_>) -> PyObject {
        self.0
    }
}

impl ToPyObject for MyPyObjectWrapper {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.0.clone_ref(py)
    }
}
```

After:
```rust,no_run
# #![allow(deprecated)]
# use pyo3::prelude::*;
# #[allow(dead_code)]
# struct MyPyObjectWrapper(PyObject);

impl<'py> IntoPyObject<'py> for MyPyObjectWrapper {
    type Target = PyAny; // the Python type
    type Output = Bound<'py, Self::Target>; // in most cases this will be `Bound`
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.into_bound(py))
    }
}

// `ToPyObject` implementations should be converted to implementations on reference types
impl<'a, 'py> IntoPyObject<'py> for &'a MyPyObjectWrapper {
    type Target = PyAny;
    type Output = Borrowed<'a, 'py, Self::Target>; // `Borrowed` can be used to optimized reference counting
    type Error = std::convert::Infallible;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        Ok(self.0.bind_borrowed(py))
    }
}
```
</details>

### To-Python conversions changed for byte collections (`Vec<u8>`, `[u8; N]` and `SmallVec<[u8; N]>`).
<details>
<summary><small>Click to expand</small></summary>

With the introduction of the `IntoPyObject` trait, PyO3's macros now prefer `IntoPyObject` implementations over `IntoPy<PyObject>` when producing Python values. This applies to `#[pyfunction]` and `#[pymethods]` return values and also fields accessed via `#[pyo3(get)]`.

This change has an effect on functions and methods returning _byte_ collections like
- `Vec<u8>`
- `[u8; N]`
- `SmallVec<[u8; N]>`

In their new `IntoPyObject` implementation these will now turn into `PyBytes` rather than a
`PyList`. All other `T`s are unaffected and still convert into a `PyList`.

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
#[pyfunction]
fn foo() -> Vec<u8> { // would previously turn into a `PyList`, now `PyBytes`
    vec![0, 1, 2, 3]
}

#[pyfunction]
fn bar() -> Vec<u16> { // unaffected, returns `PyList`
    vec![0, 1, 2, 3]
}
```

If this conversion is _not_ desired, consider building a list manually using `PyList::new`.

The following types were previously _only_ implemented for `u8` and now allow other `T`s turn into `PyList`:
- `&[T]`
- `Cow<[T]>`

This is purely additional and should just extend the possible return types.

</details>

### `gil-refs` feature removed
<details>
<summary><small>Click to expand</small></summary>

PyO3 0.23 completes the removal of the "GIL Refs" API in favour of the new "Bound" API introduced in PyO3 0.21.

With the removal of the old API, many "Bound" API functions which had been introduced with `_bound` suffixes no longer need the suffixes as these names have been freed up. For example, `PyTuple::new_bound` is now just `PyTuple::new` (the existing name remains but is deprecated).

Before:

```rust,ignore
# #![allow(deprecated)]
# use pyo3::prelude::*;
# use pyo3::types::PyTuple;
# fn main() {
# Python::attach(|py| {
// For example, for PyTuple. Many such APIs have been changed.
let tup = PyTuple::new_bound(py, [1, 2, 3]);
# })
# }
```

After:

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyTuple;
# fn main() {
# Python::attach(|py| {
// For example, for PyTuple. Many such APIs have been changed.
let tup = PyTuple::new(py, [1, 2, 3]);
# })
# }
```

#### `IntoPyDict` trait adjusted for removal of `gil-refs`

As part of this API simplification, the `IntoPyDict` trait has had a small breaking change: `IntoPyDict::into_py_dict_bound` method has been renamed to `IntoPyDict::into_py_dict`. It is also now fallible as part of the `IntoPyObject` trait addition.

If you implemented `IntoPyDict` for your type, you should implement `into_py_dict` instead of `into_py_dict_bound`. The old name is still available for calling but deprecated.

Before:

```rust,ignore
# use pyo3::prelude::*;
# use pyo3::types::{PyDict, IntoPyDict};
# use std::collections::HashMap;

struct MyMap<K, V>(HashMap<K, V>);

impl<K, V> IntoPyDict for MyMap<K, V>
where
    K: ToPyObject,
    V: ToPyObject,
{
    fn into_py_dict_bound(self, py: Python<'_>) -> Bound<'_, PyDict> {
        let dict = PyDict::new_bound(py);
        for (key, value) in self.0 {
            dict.set_item(key, value)
                .expect("Failed to set_item on dict");
        }
        dict
    }
}
```

After:

```rust,no_run
# use pyo3::prelude::*;
# use pyo3::types::{PyDict, IntoPyDict};
# use std::collections::HashMap;

# #[allow(dead_code)]
struct MyMap<K, V>(HashMap<K, V>);

impl<'py, K, V> IntoPyDict<'py> for MyMap<K, V>
where
    K: IntoPyObject<'py>,
    V: IntoPyObject<'py>,
{
    fn into_py_dict(self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        for (key, value) in self.0 {
            dict.set_item(key, value)?;
        }
        Ok(dict)
    }
}
```
</details>

## from 0.21.* to 0.22

### Deprecation of `gil-refs` feature continues
<details>
<summary><small>Click to expand</small></summary>

Following the introduction of the "Bound" API in PyO3 0.21 and the planned removal of the "GIL Refs" API, all functionality related to GIL Refs is now gated behind the `gil-refs` feature and emits a deprecation warning on use.

See <a href="#from-021-to-022">the 0.21 migration entry</a> for help upgrading.
</details>

### Deprecation of implicit default for trailing optional arguments
<details>
<summary><small>Click to expand</small></summary>

With `pyo3` 0.22 the implicit `None` default for trailing `Option<T>` type argument is deprecated. To migrate, place a `#[pyo3(signature = (...))]` attribute on affected functions or methods and specify the desired behavior.
The migration warning specifies the corresponding signature to keep the current behavior. With 0.23 the signature will be required for any function containing `Option<T>` type parameters to prevent accidental
and unnoticed changes in behavior. With 0.24 this restriction will be lifted again and `Option<T>` type arguments will be treated as any other argument _without_ special handling.

Before:

```rust,no_run
# #![allow(deprecated, dead_code)]
# use pyo3::prelude::*;
#[pyfunction]
fn increment(x: u64, amount: Option<u64>) -> u64 {
    x + amount.unwrap_or(1)
}
```

After:

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
#[pyfunction]
#[pyo3(signature = (x, amount=None))]
fn increment(x: u64, amount: Option<u64>) -> u64 {
    x + amount.unwrap_or(1)
}
```
</details>

### `Py::clone` is now gated behind the `py-clone` feature
<details>
<summary><small>Click to expand</small></summary>
If you rely on `impl<T> Clone for Py<T>` to fulfil trait requirements imposed by existing Rust code written without PyO3-based code in mind, the newly introduced feature `py-clone` must be enabled.

However, take care to note that the behaviour is different from previous versions. If `Clone` was called without the GIL being held, we tried to delay the application of these reference count increments until PyO3-based code would re-acquire it. This turned out to be impossible to implement in a sound manner and hence was removed. Now, if `Clone` is called without the GIL being held, we panic instead for which calling code might not be prepared.

It is advised to migrate off the `py-clone` feature. The simplest way to remove dependency on `impl<T> Clone for Py<T>` is to wrap `Py<T>` as `Arc<Py<T>>` and use cloning of the arc.

Related to this, we also added a `pyo3_disable_reference_pool` conditional compilation flag which removes the infrastructure necessary to apply delayed reference count decrements implied by `impl<T> Drop for Py<T>`. They do not appear to be a soundness hazard as they should lead to memory leaks in the worst case. However, the global synchronization adds significant overhead to cross the Python-Rust boundary. Enabling this feature will remove these costs and make the `Drop` implementation abort the process if called without the GIL being held instead.
</details>

### Require explicit opt-in for comparison for simple enums
<details>
<summary><small>Click to expand</small></summary>

With `pyo3` 0.22 the new `#[pyo3(eq)]` options allows automatic implementation of Python equality using Rust's `PartialEq`. Previously simple enums automatically implemented equality in terms of their discriminants. To make PyO3 more consistent, this automatic equality implementation is deprecated in favour of having opt-ins for all `#[pyclass]` types. Similarly, simple enums supported comparison with integers, which is not covered by Rust's `PartialEq` derive, so has been split out into the `#[pyo3(eq_int)]` attribute.

To migrate, place a `#[pyo3(eq, eq_int)]` attribute on simple enum classes.

Before:

```rust,no_run
# #![allow(deprecated, dead_code)]
# use pyo3::prelude::*;
#[pyclass]
enum SimpleEnum {
    VariantA,
    VariantB = 42,
}
```

After:

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
#[pyclass(eq, eq_int)]
#[derive(PartialEq)]
enum SimpleEnum {
    VariantA,
    VariantB = 42,
}
```
</details>

### `PyType::name` reworked to better match Python `__name__`
<details>
<summary><small>Click to expand</small></summary>

This function previously would try to read directly from Python type objects' C API field (`tp_name`), in which case it
would return a `Cow::Borrowed`. However the contents of `tp_name` don't have well-defined semantics.

Instead `PyType::name()` now returns the equivalent of Python `__name__` and returns `PyResult<Bound<'py, PyString>>`.

The closest equivalent to PyO3 0.21's version of `PyType::name()` has been introduced as a new function `PyType::fully_qualified_name()`,
which is equivalent to `__module__` and `__qualname__` joined as `module.qualname`.

Before:

```rust,ignore
# #![allow(deprecated, dead_code)]
# use pyo3::prelude::*;
# use pyo3::types::{PyBool};
# fn main() -> PyResult<()> {
Python::with_gil(|py| {
    let bool_type = py.get_type_bound::<PyBool>();
    let name = bool_type.name()?.into_owned();
    println!("Hello, {}", name);

    let mut name_upper = bool_type.name()?;
    name_upper.to_mut().make_ascii_uppercase();
    println!("Hello, {}", name_upper);

    Ok(())
})
# }
```

After:

```rust,ignore
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::types::{PyBool};
# fn main() -> PyResult<()> {
Python::with_gil(|py| {
    let bool_type = py.get_type_bound::<PyBool>();
    let name = bool_type.name()?;
    println!("Hello, {}", name);

    // (if the full dotted path was desired, switch from `name()` to `fully_qualified_name()`)
    let mut name_upper = bool_type.fully_qualified_name()?.to_string();
    name_upper.make_ascii_uppercase();
    println!("Hello, {}", name_upper);

    Ok(())
})
# }
```
</details>



## from 0.20.* to 0.21
<details>
<summary><small>Click to expand</small></summary>

PyO3 0.21 introduces a new `Bound<'py, T>` smart pointer which replaces the existing "GIL Refs" API to interact with Python objects. For example, in PyO3 0.20 the reference `&'py PyAny` would be used to interact with Python objects. In PyO3 0.21 the updated type is `Bound<'py, PyAny>`. Making this change moves Rust ownership semantics out of PyO3's internals and into user code. This change fixes [a known soundness edge case of interaction with gevent](https://github.com/PyO3/pyo3/issues/3668) as well as improves CPU and [memory performance](https://github.com/PyO3/pyo3/issues/1056). For a full history of discussion see https://github.com/PyO3/pyo3/issues/3382.

The "GIL Ref" `&'py PyAny` and similar types such as `&'py PyDict` continue to be available as a deprecated API. Due to the advantages of the new API it is advised that all users make the effort to upgrade as soon as possible.

In addition to the major API type overhaul, PyO3 has needed to make a few small breaking adjustments to other APIs to close correctness and soundness gaps.

The recommended steps to update to PyO3 0.21 is as follows:
  1. Enable the `gil-refs` feature to silence deprecations related to the API change
  2. Fix all other PyO3 0.21 migration steps
  3. Disable the `gil-refs` feature and migrate off the deprecated APIs

The following sections are laid out in this order.
</details>

### Enable the `gil-refs` feature
<details>
<summary><small>Click to expand</small></summary>

To make the transition for the PyO3 ecosystem away from the GIL Refs API as smooth as possible, in PyO3 0.21 no APIs consuming or producing GIL Refs have been altered. Instead, variants using `Bound<T>` smart pointers have been introduced, for example `PyTuple::new_bound` which returns `Bound<PyTuple>` is the replacement form of `PyTuple::new`. The GIL Ref APIs have been deprecated, but to make migration easier it is possible to disable these deprecation warnings by enabling the `gil-refs` feature.

> The one single exception where an existing API was changed in-place is the `pyo3::intern!` macro. Almost all uses of this macro did not need to update code to account it changing to return `&Bound<PyString>` immediately, and adding an `intern_bound!` replacement was perceived as adding more work for users.

It is recommended that users do this as a first step of updating to PyO3 0.21 so that the deprecation warnings do not get in the way of resolving the rest of the migration steps.

Before:

```toml
# Cargo.toml
[dependencies]
pyo3 = "0.20"
```

After:

```toml
# Cargo.toml
[dependencies]
pyo3 = { version = "0.21", features = ["gil-refs"] }
```
</details>

### `PyTypeInfo` and `PyTryFrom` have been adjusted
<details>
<summary><small>Click to expand</small></summary>

The `PyTryFrom` trait has aged poorly, its `try_from` method now conflicts with `TryFrom::try_from` in the 2021 edition prelude. A lot of its functionality was also duplicated with `PyTypeInfo`.

To tighten up the PyO3 traits as part of the deprecation of the GIL Refs API the `PyTypeInfo` trait has had a simpler companion `PyTypeCheck`. The methods `PyAny::downcast` and `PyAny::downcast_exact` no longer use `PyTryFrom` as a bound, instead using `PyTypeCheck` and `PyTypeInfo` respectively.

To migrate, switch all type casts to use `obj.downcast()` instead of `try_from(obj)` (and similar for `downcast_exact`).

Before:

```rust,ignore
# #![allow(deprecated)]
# use pyo3::prelude::*;
# use pyo3::types::{PyInt, PyList};
# fn main() -> PyResult<()> {
Python::with_gil(|py| {
    let list = PyList::new(py, 0..5);
    let b = <PyInt as PyTryFrom>::try_from(list.get_item(0).unwrap())?;
    Ok(())
})
# }
```

After:

```rust,ignore
# use pyo3::prelude::*;
# use pyo3::types::{PyInt, PyList};
# fn main() -> PyResult<()> {
Python::with_gil(|py| {
    // Note that PyList::new is deprecated for PyList::new_bound as part of the GIL Refs API removal,
    // see the section below on migration to Bound<T>.
    #[allow(deprecated)]
    let list = PyList::new(py, 0..5);
    let b = list.get_item(0).unwrap().downcast::<PyInt>()?;
    Ok(())
})
# }
```
</details>

### `Iter(A)NextOutput` are deprecated
<details>
<summary><small>Click to expand</small></summary>

The `__next__` and `__anext__` magic methods can now return any type convertible into Python objects directly just like all other `#[pymethods]`. The `IterNextOutput` used by `__next__` and `IterANextOutput` used by `__anext__` are subsequently deprecated. Most importantly, this change allows returning an awaitable from `__anext__` without non-sensically wrapping it into `Yield` or `Some`. Only the return types `Option<T>` and `Result<Option<T>, E>` are still handled in a special manner where `Some(val)` yields `val` and `None` stops iteration.

Starting with an implementation of a Python iterator using `IterNextOutput`, e.g.

```rust,ignore
use pyo3::prelude::*;
use pyo3::iter::IterNextOutput;

#[pyclass]
struct PyClassIter {
    count: usize,
}

#[pymethods]
impl PyClassIter {
    fn __next__(&mut self) -> IterNextOutput<usize, &'static str> {
        if self.count < 5 {
            self.count += 1;
            IterNextOutput::Yield(self.count)
        } else {
            IterNextOutput::Return("done")
        }
    }
}
```

If returning `"done"` via `StopIteration` is not really required, this should be written as

```rust,no_run
use pyo3::prelude::*;

#[pyclass]
struct PyClassIter {
    count: usize,
}

#[pymethods]
impl PyClassIter {
    fn __next__(&mut self) -> Option<usize> {
        if self.count < 5 {
            self.count += 1;
            Some(self.count)
        } else {
            None
        }
    }
}
```

This form also has additional benefits: It has already worked in previous PyO3 versions, it matches the signature of Rust's [`Iterator` trait](https://doc.rust-lang.org/stable/std/iter/trait.Iterator.html) and it allows using a fast path in CPython which completely avoids the cost of raising a `StopIteration` exception. Note that using [`Option::transpose`](https://doc.rust-lang.org/stable/std/option/enum.Option.html#method.transpose) and the `Result<Option<T>, E>` variant, this form can also be used to wrap fallible iterators.

Alternatively, the implementation can also be done as it would in Python itself, i.e. by "raising" a `StopIteration` exception

```rust,no_run
use pyo3::prelude::*;
use pyo3::exceptions::PyStopIteration;

#[pyclass]
struct PyClassIter {
    count: usize,
}

#[pymethods]
impl PyClassIter {
    fn __next__(&mut self) -> PyResult<usize> {
        if self.count < 5 {
            self.count += 1;
            Ok(self.count)
        } else {
            Err(PyStopIteration::new_err("done"))
        }
    }
}
```

Finally, an asynchronous iterator can directly return an awaitable without confusing wrapping

```rust,no_run
use pyo3::prelude::*;

#[pyclass]
struct PyClassAwaitable {
    number: usize,
}

#[pymethods]
impl PyClassAwaitable {
    fn __next__(&self) -> usize {
        self.number
    }

    fn __await__(slf: Py<Self>) -> Py<Self> {
        slf
    }
}

#[pyclass]
struct PyClassAsyncIter {
    number: usize,
}

#[pymethods]
impl PyClassAsyncIter {
    fn __anext__(&mut self) -> PyClassAwaitable {
        self.number += 1;
        PyClassAwaitable {
            number: self.number,
        }
    }

    fn __aiter__(slf: Py<Self>) -> Py<Self> {
        slf
    }
}
```
</details>

### `PyType::name` has been renamed to `PyType::qualname`
<details>
<summary><small>Click to expand</small></summary>

`PyType::name` has been renamed to `PyType::qualname` to indicate that it does indeed return the [qualified name](https://docs.python.org/3/glossary.html#term-qualified-name), matching the `__qualname__` attribute. The newly added `PyType::name` yields the full name including the module name now which corresponds to `__module__.__name__` on the level of attributes.
</details>

### `PyCell` has been deprecated
<details>
<summary><small>Click to expand</small></summary>

Interactions with Python objects implemented in Rust no longer need to go though `PyCell<T>`. Instead interactions with Python object now consistently go through `Bound<T>` or `Py<T>` independently of whether `T` is native Python object or a `#[pyclass]` implemented in Rust. Use `Bound::new` or `Py::new` respectively to create and `Bound::borrow(_mut)` / `Py::borrow(_mut)` to borrow the Rust object.
</details>

### Migrating from the GIL Refs API to `Bound<T>`
<details>
<summary><small>Click to expand</small></summary>

To minimise breakage of code using the GIL Refs API, the `Bound<T>` smart pointer has been introduced by adding complements to all functions which accept or return GIL Refs. This allows code to migrate by replacing the deprecated APIs with the new ones.

To identify what to migrate, temporarily switch off the `gil-refs` feature to see deprecation warnings on [almost](#cases-where-pyo3-cannot-emit-gil-ref-deprecation-warnings) all uses of APIs accepting and producing GIL Refs . Over one or more PRs it should be possible to follow the deprecation hints to update code. Depending on your development environment, switching off the `gil-refs` feature may introduce [some very targeted breakages](#deactivating-the-gil-refs-feature), so you may need to fixup those first.

For example, the following APIs have gained updated variants:
- `PyList::new`, `PyTuple::new` and similar constructors have replacements `PyList::new_bound`, `PyTuple::new_bound` etc.
- `FromPyObject::extract` has a new `FromPyObject::extract_bound` (see the section below)
- The `PyTypeInfo` trait has had new `_bound` methods added to accept / return `Bound<T>`.

Because the new `Bound<T>` API brings ownership out of the PyO3 framework and into user code, there are a few places where user code is expected to need to adjust while switching to the new API:
- Code will need to add the occasional `&` to borrow the new smart pointer as `&Bound<T>` to pass these types around (or use `.clone()` at the very small cost of increasing the Python reference count)
- `Bound<PyList>` and `Bound<PyTuple>` cannot support indexing with `list[0]`, you should use `list.get_item(0)` instead.
- `Bound<PyTuple>::iter_borrowed` is slightly more efficient than `Bound<PyTuple>::iter`. The default iteration of `Bound<PyTuple>` cannot return borrowed references because Rust does not (yet) have "lending iterators". Similarly `Bound<PyTuple>::get_borrowed_item` is more efficient than `Bound<PyTuple>::get_item` for the same reason.
- `&Bound<T>` does not implement `FromPyObject` (although it might be possible to do this in the future once the GIL Refs API is completely removed). Use `bound_any.downcast::<T>()` instead of `bound_any.extract::<&Bound<T>>()`.
- `Bound<PyString>::to_str` now borrows from the `Bound<PyString>` rather than from the `'py` lifetime, so code will need to store the smart pointer as a value in some cases where previously `&PyString` was just used as a temporary. (There are some more details relating to this in [the section below](#deactivating-the-gil-refs-feature).)
- `.extract::<&str>()` now borrows from the source Python object. The simplest way to update is to change to `.extract::<PyBackedStr>()`, which retains ownership of the Python reference. See more information [in the section on deactivating the `gil-refs` feature](#deactivating-the-gil-refs-feature).

To convert between `&PyAny` and `&Bound<PyAny>` use the `as_borrowed()` method:

```rust,ignore
let gil_ref: &PyAny = ...;
let bound: &Bound<PyAny> = &gil_ref.as_borrowed();
```

To convert between `Py<T>` and `Bound<T>` use the `bind()` / `into_bound()` methods, and `as_unbound()` / `unbind()` to go back from `Bound<T>` to `Py<T>`.

```rust,ignore
let obj: Py<PyList> = ...;
let bound: &Bound<'py, PyList> = obj.bind(py);
let bound: Bound<'py, PyList> = obj.into_bound(py);

let obj: &Py<PyList> = bound.as_unbound();
let obj: Py<PyList> = bound.unbind();
```

<div class="warning">

⚠️ Warning: dangling pointer trap 💣

> Because of the ownership changes, code which uses `.as_ptr()` to convert `&PyAny` and other GIL Refs to a `*mut pyo3_ffi::PyObject` should take care to avoid creating dangling pointers now that `Bound<PyAny>` carries ownership.
>
> For example, the following pattern with `Option<&PyAny>` can easily create a dangling pointer when migrating to the `Bound<PyAny>` smart pointer:
>
> ```rust,ignore
> let opt: Option<&PyAny> = ...;
> let p: *mut ffi::PyObject = opt.map_or(std::ptr::null_mut(), |any| any.as_ptr());
> ```
>
> The correct way to migrate this code is to use `.as_ref()` to avoid dropping the `Bound<PyAny>` in the `map_or` closure:
>
> ```rust,ignore
> let opt: Option<Bound<PyAny>> = ...;
> let p: *mut ffi::PyObject = opt.as_ref().map_or(std::ptr::null_mut(), Bound::as_ptr);
> ```
<div>

#### Migrating `FromPyObject` implementations

`FromPyObject` has had a new method `extract_bound` which takes `&Bound<'py, PyAny>` as an argument instead of `&PyAny`. Both `extract` and `extract_bound` have been given default implementations in terms of the other, to avoid breaking code immediately on update to 0.21.

All implementations of `FromPyObject` should be switched from `extract` to `extract_bound`.

Before:

```rust,ignore
impl<'py> FromPyObject<'py> for MyType {
    fn extract(obj: &'py PyAny) -> PyResult<Self> {
        /* ... */
    }
}
```

After:

```rust,ignore
impl<'py> FromPyObject<'py> for MyType {
    fn extract_bound(obj: &Bound<'py, PyAny>) -> PyResult<Self> {
        /* ... */
    }
}
```

The expectation is that in 0.22 `extract_bound` will have the default implementation removed and in 0.23 `extract` will be removed.

#### Cases where PyO3 cannot emit GIL Ref deprecation warnings

Despite a large amount of deprecations warnings produced by PyO3 to aid with the transition from GIL Refs to the Bound API, there are a few cases where PyO3 cannot automatically warn on uses of GIL Refs. It is worth checking for these cases manually after the deprecation warnings have all been addressed:

- Individual implementations of the `FromPyObject` trait cannot be deprecated, so PyO3 cannot warn about uses of code patterns like `.extract<&PyAny>()` which produce a GIL Ref.
- GIL Refs in `#[pyfunction]` arguments emit a warning, but if the GIL Ref is wrapped inside another container such as `Vec<&PyAny>` then PyO3 cannot warn against this.
- The `wrap_pyfunction!(function)(py)` deferred argument form of the `wrap_pyfunction` macro taking `py: Python<'py>` produces a GIL Ref, and due to limitations in type inference PyO3 cannot warn against this specific case.

</details>

### Deactivating the `gil-refs` feature
<details>
<summary><small>Click to expand</small></summary>

As a final step of migration, deactivating the `gil-refs` feature will set up code for best performance and is intended to set up a forward-compatible API for PyO3 0.22.

At this point code that needed to manage GIL Ref memory can safely remove uses of `GILPool` (which are constructed by calls to `Python::new_pool` and `Python::with_pool`). Deprecation warnings will highlight these cases.

There is just one case of code that changes upon disabling these features: `FromPyObject` trait implementations for types that borrow directly from the input data cannot be implemented by PyO3 without GIL Refs (while the GIL Refs API is in the process of being removed). The main types affected are `&str`, `Cow<'_, str>`, `&[u8]`, `Cow<'_, u8>`.

To make PyO3's core functionality continue to work while the GIL Refs API is in the process of being removed, disabling the `gil-refs` feature moves the implementations of `FromPyObject` for `&str`, `Cow<'_, str>`, `&[u8]`, `Cow<'_, u8>` to a new temporary trait `FromPyObjectBound`. This trait is the expected future form of `FromPyObject` and has an additional lifetime `'a` to enable these types to borrow data from Python objects.

PyO3 0.21 has introduced the [`PyBackedStr`]({{#PYO3_DOCS_URL}}/pyo3/pybacked/struct.PyBackedStr.html) and [`PyBackedBytes`]({{#PYO3_DOCS_URL}}/pyo3/pybacked/struct.PyBackedBytes.html) types to help with this case. The easiest way to avoid lifetime challenges from extracting `&str` is to use these. For more complex types like `Vec<&str>`, is now impossible to extract directly from a Python object and `Vec<PyBackedStr>` is the recommended upgrade path.

A key thing to note here is because extracting to these types now ties them to the input lifetime, some extremely common patterns may need to be split into multiple Rust lines. For example, the following snippet of calling `.extract::<&str>()` directly on the result of `.getattr()` needs to be adjusted when deactivating the `gil-refs` feature.

Before:

```rust,ignore
# #[cfg(feature = "gil-refs")] {
# use pyo3::prelude::*;
# use pyo3::types::{PyList, PyType};
# fn example<'py>(py: Python<'py>) -> PyResult<()> {
#[allow(deprecated)] // GIL Ref API
let obj: &'py PyType = py.get_type::<PyList>();
let name: &'py str = obj.getattr("__name__")?.extract()?;
assert_eq!(name, "list");
# Ok(())
# }
# Python::with_gil(example).unwrap();
# }
```

After:

```rust,ignore
# #[cfg(any(not(Py_LIMITED_API), Py_3_10))] {
# use pyo3::prelude::*;
# use pyo3::types::{PyList, PyType};
# fn example<'py>(py: Python<'py>) -> PyResult<()> {
let obj: Bound<'py, PyType> = py.get_type_bound::<PyList>();
let name_obj: Bound<'py, PyAny> = obj.getattr("__name__")?;
// the lifetime of the data is no longer `'py` but the much shorter
// lifetime of the `name_obj` smart pointer above
let name: &'_ str = name_obj.extract()?;
assert_eq!(name, "list");
# Ok(())
# }
# Python::with_gil(example).unwrap();
# }
```

To avoid needing to worry about lifetimes at all, it is also possible to use the new `PyBackedStr` type, which stores a reference to the Python `str` without a lifetime attachment. In particular, `PyBackedStr` helps for `abi3` builds for Python older than 3.10. Due to limitations in the `abi3` CPython API for those older versions, PyO3 cannot offer a `FromPyObjectBound` implementation for `&str` on those versions. The easiest way to migrate for older `abi3` builds is to replace any cases of `.extract::<&str>()` with `.extract::<PyBackedStr>()`. Alternatively, use `.extract::<Cow<str>>()`, `.extract::<String>()` to copy the data into Rust.

The following example uses the same snippet as those just above, but this time the final extracted type is `PyBackedStr`:

```rust,ignore
# use pyo3::prelude::*;
# use pyo3::types::{PyList, PyType};
# fn example<'py>(py: Python<'py>) -> PyResult<()> {
use pyo3::pybacked::PyBackedStr;
let obj: Bound<'py, PyType> = py.get_type_bound::<PyList>();
let name: PyBackedStr = obj.getattr("__name__")?.extract()?;
assert_eq!(&*name, "list");
# Ok(())
# }
# Python::with_gil(example).unwrap();
```
</details>

## from 0.19.* to 0.20

### Drop support for older technologies
<details>
<summary><small>Click to expand</small></summary>

PyO3 0.20 has increased minimum Rust version to 1.56. This enables use of newer language features and simplifies maintenance of the project.
</details>

### `PyDict::get_item` now returns a `Result`
<details>
<summary><small>Click to expand</small></summary>

`PyDict::get_item` in PyO3 0.19 and older was implemented using a Python API which would suppress all exceptions and return `None` in those cases. This included errors in `__hash__` and `__eq__` implementations of the key being looked up.

Newer recommendations by the Python core developers advise against using these APIs which suppress exceptions, instead allowing exceptions to bubble upwards. `PyDict::get_item_with_error` already implemented this recommended behavior, so that API has been renamed to `PyDict::get_item`.


Before:

```rust,ignore
use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyDict, IntoPyDict};

# fn main() {
# let _ =
Python::with_gil(|py| {
    let dict: &PyDict = [("a", 1)].into_py_dict(py);
    // `a` is in the dictionary, with value 1
    assert!(dict.get_item("a").map_or(Ok(false), |x| x.eq(1))?);
    // `b` is not in the dictionary
    assert!(dict.get_item("b").is_none());
    // `dict` is not hashable, so this fails with a `TypeError`
    assert!(dict
        .get_item_with_error(dict)
        .unwrap_err()
        .is_instance_of::<PyTypeError>(py));
});
# }
```

After:

```rust,ignore
use pyo3::prelude::*;
use pyo3::exceptions::PyTypeError;
use pyo3::types::{PyDict, IntoPyDict};

# fn main() {
# let _ =
Python::with_gil(|py| -> PyResult<()> {
    let dict: &PyDict = [("a", 1)].into_py_dict(py);
    // `a` is in the dictionary, with value 1
    assert!(dict.get_item("a")?.map_or(Ok(false), |x| x.eq(1))?);
    // `b` is not in the dictionary
    assert!(dict.get_item("b")?.is_none());
    // `dict` is not hashable, so this fails with a `TypeError`
    assert!(dict
        .get_item(dict)
        .unwrap_err()
        .is_instance_of::<PyTypeError>(py));

    Ok(())
});
# }
```
</details>

### Required arguments are no longer accepted after optional arguments
<details>
<summary><small>Click to expand</small></summary>

Trailing `Option<T>` arguments have an automatic default of `None`. To avoid unwanted changes when modifying function signatures, in PyO3 0.18 it was deprecated to have a required argument after an `Option<T>` argument without using `#[pyo3(signature = (...))]` to specify the intended defaults. In PyO3 0.20, this becomes a hard error.

Before:

```rust,ignore
#[pyfunction]
fn x_or_y(x: Option<u64>, y: u64) -> u64 {
    x.unwrap_or(y)
}
```

After:

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (x, y))] // both x and y have no defaults and are required
fn x_or_y(x: Option<u64>, y: u64) -> u64 {
    x.unwrap_or(y)
}
```
</details>

### Remove deprecated function forms
<details>
<summary><small>Click to expand</small></summary>

In PyO3 0.18 the `#[args]` attribute for `#[pymethods]`, and directly specifying the function signature in `#[pyfunction]`, was deprecated. This functionality has been removed in PyO3 0.20.

Before:

```rust,ignore
#[pyfunction]
#[pyo3(a, b = "0", "/")]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
```

After:

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (a, b=0, /))]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
```

</details>

### `IntoPyPointer` trait removed
<details>
<summary><small>Click to expand</small></summary>

The trait `IntoPyPointer`, which provided the `into_ptr` method on many types, has been removed. `into_ptr` is now available as an inherent method on all types that previously implemented this trait.
</details>

### `AsPyPointer` now `unsafe` trait
<details>
<summary><small>Click to expand</small></summary>

The trait `AsPyPointer` is now `unsafe trait`, meaning any external implementation of it must be marked as `unsafe impl`, and ensure that they uphold the invariant of returning valid pointers.
</details>

## from 0.18.* to 0.19

### Access to `Python` inside `__traverse__` implementations are now forbidden
<details>
<summary><small>Click to expand</small></summary>

During `__traverse__` implementations for Python's Garbage Collection it is forbidden to do anything other than visit the members of the `#[pyclass]` being traversed. This means making Python function calls or other API calls are forbidden.

Previous versions of PyO3 would allow access to `Python` (e.g. via `Python::with_gil`), which could cause the Python interpreter to crash or otherwise confuse the garbage collection algorithm.

Attempts to acquire the GIL will now panic. See [#3165](https://github.com/PyO3/pyo3/issues/3165) for more detail.

```rust,ignore
# use pyo3::prelude::*;

#[pyclass]
struct SomeClass {}

impl SomeClass {
    fn __traverse__(&self, pyo3::class::gc::PyVisit<'_>) -> Result<(), pyo3::class::gc::PyTraverseError>` {
        Python::with_gil(|| { /*...*/ })  // ERROR: this will panic
    }
}
```
</details>

### Smarter `anyhow::Error` / `eyre::Report` conversion when inner error is "simple" `PyErr`
<details>
<summary><small>Click to expand</small></summary>

When converting from `anyhow::Error` or `eyre::Report` to `PyErr`, if the inner error is a "simple" `PyErr` (with no source error), then the inner error will be used directly as the `PyErr` instead of wrapping it in a new `PyRuntimeError` with the original information converted into a string.

```rust,ignore
# #[cfg(feature = "anyhow")]
# #[allow(dead_code)]
# mod anyhow_only {
# use pyo3::prelude::*;
# use pyo3::exceptions::PyValueError;
#[pyfunction]
fn raise_err() -> anyhow::Result<()> {
    Err(PyValueError::new_err("original error message").into())
}

fn main() {
    Python::with_gil(|py| {
        let rs_func = wrap_pyfunction!(raise_err, py).unwrap();
        pyo3::py_run!(
            py,
            rs_func,
            r"
        try:
            rs_func()
        except Exception as e:
            print(repr(e))
        "
        );
    })
}
# }
```

Before, the above code would have printed `RuntimeError('ValueError: original error message')`, which might be confusing.

After, the same code will print `ValueError: original error message`, which is more straightforward.

However, if the `anyhow::Error` or `eyre::Report` has a source, then the original exception will still be wrapped in a `PyRuntimeError`.
</details>

### The deprecated `Python::acquire_gil` was removed and `Python::with_gil` must be used instead
<details>
<summary><small>Click to expand</small></summary>

While the API provided by [`Python::acquire_gil`](https://docs.rs/pyo3/0.18.3/pyo3/marker/struct.Python.html#method.acquire_gil) seems convenient, it is somewhat brittle as the design of the GIL token [`Python`](https://docs.rs/pyo3/0.18.3/pyo3/marker/struct.Python.html) relies on proper nesting and panics if not used correctly, e.g.

```rust,ignore
# #![allow(dead_code, deprecated)]
# use pyo3::prelude::*;

#[pyclass]
struct SomeClass {}

struct ObjectAndGuard {
    object: Py<SomeClass>,
    guard: GILGuard,
}

impl ObjectAndGuard {
    fn new() -> Self {
        let guard = Python::acquire_gil();
        let object = Py::new(guard.python(), SomeClass {}).unwrap();

        Self { object, guard }
    }
}

let first = ObjectAndGuard::new();
let second = ObjectAndGuard::new();
// Panics because the guard within `second` is still alive.
drop(first);
drop(second);
```

The replacement is [`Python::with_gil`](https://docs.rs/pyo3/0.18.3/pyo3/marker/struct.Python.html#method.with_gil) which is more cumbersome but enforces the proper nesting by design, e.g.

```rust,ignore
# #![allow(dead_code)]
# use pyo3::prelude::*;

#[pyclass]
struct SomeClass {}

struct Object {
    object: Py<SomeClass>,
}

impl Object {
    fn new(py: Python<'_>) -> Self {
        let object = Py::new(py, SomeClass {}).unwrap();

        Self { object }
    }
}

// It either forces us to release the GIL before acquiring it again.
let first = Python::with_gil(|py| Object::new(py));
let second = Python::with_gil(|py| Object::new(py));
drop(first);
drop(second);

// Or it ensures releasing the inner lock before the outer one.
Python::with_gil(|py| {
    let first = Object::new(py);
    let second = Python::with_gil(|py| Object::new(py));
    drop(first);
    drop(second);
});
```

Furthermore, `Python::acquire_gil` provides ownership of a `GILGuard` which can be freely stored and passed around. This is usually not helpful as it may keep the lock held for a long time thereby blocking progress in other parts of the program. Due to the generative lifetime attached to the GIL token supplied by `Python::with_gil`, the problem is avoided as the GIL token can only be passed down the call chain. Often, this issue can also be avoided entirely as any GIL-bound reference `&'py PyAny` implies access to a GIL token `Python<'py>` via the [`PyAny::py`](https://docs.rs/pyo3/0.22.5/pyo3/types/struct.PyAny.html#method.py) method.
</details>

## from 0.17.* to 0.18

### Required arguments after `Option<_>` arguments will no longer be automatically inferred
<details>
<summary><small>Click to expand</small></summary>

In `#[pyfunction]` and `#[pymethods]`, if a "required" function input such as `i32` came after an `Option<_>` input, then the `Option<_>` would be implicitly treated as required. (All trailing `Option<_>` arguments were treated as optional with a default value of `None`).

Starting with PyO3 0.18, this is deprecated and a future PyO3 version will require a [`#[pyo3(signature = (...))]` option](./function/signature.md) to explicitly declare the programmer's intention.

Before, x in the below example would be required to be passed from Python code:

```rust,compile_fail,ignore
# #![allow(dead_code)]
# use pyo3::prelude::*;

#[pyfunction]
fn required_argument_after_option(x: Option<i32>, y: i32) {}
```

After, specify the intended Python signature explicitly:

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;

// If x really was intended to be required
#[pyfunction(signature = (x, y))]
fn required_argument_after_option_a(x: Option<i32>, y: i32) {}

// If x was intended to be optional, y needs a default too
#[pyfunction(signature = (x=None, y=0))]
fn required_argument_after_option_b(x: Option<i32>, y: i32) {}
```
</details>

### `__text_signature__` is now automatically generated for `#[pyfunction]` and `#[pymethods]`
<details>
<summary><small>Click to expand</small></summary>

The [`#[pyo3(text_signature = "...")]` option](./function/signature.md#making-the-function-signature-available-to-python) was previously the only supported way to set the `__text_signature__` attribute on generated Python functions.

PyO3 is now able to automatically populate `__text_signature__` for all functions automatically based on their Rust signature (or the [new `#[pyo3(signature = (...))]` option](./function/signature.md)). These automatically-generated `__text_signature__` values will currently only render `...` for all default values. Many `#[pyo3(text_signature = "...")]` options can be removed from functions when updating to PyO3 0.18, however in cases with default values a manual implementation may still be preferred for now.

As examples:

```rust
# use pyo3::prelude::*;

// The `text_signature` option here is no longer necessary, as PyO3 will automatically
// generate exactly the same value.
#[pyfunction(text_signature = "(a, b, c)")]
fn simple_function(a: i32, b: i32, c: i32) {}

// The `text_signature` still provides value here as of PyO3 0.18, because the automatically
// generated signature would be "(a, b=..., c=...)".
#[pyfunction(signature = (a, b = 1, c = 2), text_signature = "(a, b=1, c=2)")]
fn function_with_defaults(a: i32, b: i32, c: i32) {}

# fn main() {
#     Python::attach(|py| {
#         let simple = wrap_pyfunction!(simple_function, py).unwrap();
#         assert_eq!(simple.getattr("__text_signature__").unwrap().to_string(), "(a, b, c)");
#         let defaulted = wrap_pyfunction!(function_with_defaults, py).unwrap();
#         assert_eq!(defaulted.getattr("__text_signature__").unwrap().to_string(), "(a, b=1, c=2)");
#     })
# }
```
</details>

## from 0.16.* to 0.17

### Type checks have been changed for `PyMapping` and `PySequence` types
<details>
<summary><small>Click to expand</small></summary>

Previously the type checks for `PyMapping` and `PySequence` (implemented in `PyTryFrom`)
used the Python C-API functions `PyMapping_Check` and `PySequence_Check`.
Unfortunately these functions are not sufficient for distinguishing such types,
leading to inconsistent behavior (see
[pyo3/pyo3#2072](https://github.com/PyO3/pyo3/issues/2072)).

PyO3 0.17 changes these downcast checks to explicitly test if the type is a
subclass of the corresponding abstract base class `collections.abc.Mapping` or
`collections.abc.Sequence`. Note this requires calling into Python, which may
incur a performance penalty over the previous method. If this performance
penalty is a problem, you may be able to perform your own checks and use
`try_from_unchecked` (unsafe).

Another side-effect is that a pyclass defined in Rust with PyO3 will need to
be _registered_ with the corresponding Python abstract base class for
downcasting to succeed. `PySequence::register` and `PyMapping:register` have
been added to make it easy to do this from Rust code. These are equivalent to
calling `collections.abc.Mapping.register(MappingPyClass)` or
`collections.abc.Sequence.register(SequencePyClass)` from Python.

For example, for a mapping class defined in Rust:
```rust,compile_fail
use pyo3::prelude::*;
use std::collections::HashMap;

#[pyclass(mapping)]
struct Mapping {
    index: HashMap<String, usize>,
}

#[pymethods]
impl Mapping {
    #[new]
    fn new(elements: Option<&PyList>) -> PyResult<Self> {
    // ...
    // truncated implementation of this mapping pyclass - basically a wrapper around a HashMap
}
```

You must register the class with `collections.abc.Mapping` before the downcast will work:
```rust,compile_fail
let m = Py::new(py, Mapping { index }).unwrap();
assert!(m.as_ref(py).downcast::<PyMapping>().is_err());
PyMapping::register::<Mapping>(py).unwrap();
assert!(m.as_ref(py).downcast::<PyMapping>().is_ok());
```

Note that this requirement may go away in the future when a pyclass is able to inherit from the abstract base class directly (see [pyo3/pyo3#991](https://github.com/PyO3/pyo3/issues/991)).
</details>

### The `multiple-pymethods` feature now requires Rust 1.62
<details>
<summary><small>Click to expand</small></summary>

Due to limitations in the `inventory` crate which the `multiple-pymethods` feature depends on, this feature now
requires Rust 1.62. For more information see [dtolnay/inventory#32](https://github.com/dtolnay/inventory/issues/32).
</details>

### Added `impl IntoPy<Py<PyString>> for &str`
<details>
<summary><small>Click to expand</small></summary>

This may cause inference errors.

Before:
```rust,compile_fail
# use pyo3::prelude::*;
#
# fn main() {
Python::with_gil(|py| {
    // Cannot infer either `Py<PyAny>` or `Py<PyString>`
    let _test = "test".into_py(py);
});
# }
```

After, some type annotations may be necessary:

```rust,ignore
# #![allow(deprecated)]
# use pyo3::prelude::*;
#
# fn main() {
Python::with_gil(|py| {
    let _test: Py<PyAny> = "test".into_py(py);
});
# }
```
</details>

### The `pyproto` feature is now disabled by default
<details>
<summary><small>Click to expand</small></summary>

In preparation for removing the deprecated `#[pyproto]` attribute macro in a future PyO3 version, it is now gated behind an opt-in feature flag. This also gives a slight saving to compile times for code which does not use the deprecated macro.
</details>

### `PyTypeObject` trait has been deprecated
<details>
<summary><small>Click to expand</small></summary>

The `PyTypeObject` trait already was near-useless; almost all functionality was already on the `PyTypeInfo` trait, which `PyTypeObject` had a blanket implementation based upon. In PyO3 0.17 the final method, `PyTypeObject::type_object` was moved to `PyTypeInfo::type_object`.

To migrate, update trait bounds and imports from `PyTypeObject` to `PyTypeInfo`.

Before:

```rust,ignore
use pyo3::Python;
use pyo3::type_object::PyTypeObject;
use pyo3::types::PyType;

fn get_type_object<T: PyTypeObject>(py: Python<'_>) -> &PyType {
    T::type_object(py)
}
```

After

```rust,ignore
use pyo3::{Python, PyTypeInfo};
use pyo3::types::PyType;

fn get_type_object<T: PyTypeInfo>(py: Python<'_>) -> &PyType {
    T::type_object(py)
}

# Python::with_gil(|py| { get_type_object::<pyo3::types::PyList>(py); });
```
</details>

### `impl<T, const N: usize> IntoPy<PyObject> for [T; N]` now requires `T: IntoPy` rather than `T: ToPyObject`
<details>
<summary><small>Click to expand</small></summary>

If this leads to errors, simply implement `IntoPy`. Because pyclasses already implement `IntoPy`, you probably don't need to worry about this.
</details>

### Each `#[pymodule]` can now only be initialized once per process
<details>
<summary><small>Click to expand</small></summary>

To make PyO3 modules sound in the presence of Python sub-interpreters, for now it has been necessary to explicitly disable the ability to initialize a `#[pymodule]` more than once in the same process. Attempting to do this will now raise an `ImportError`.
</details>

## from 0.15.* to 0.16

### Drop support for older technologies
<details>
<summary><small>Click to expand</small></summary>

PyO3 0.16 has increased minimum Rust version to 1.48 and minimum Python version to 3.7. This enables use of newer language features (enabling some of the other additions in 0.16) and simplifies maintenance of the project.
</details>

### `#[pyproto]` has been deprecated
<details>
<summary><small>Click to expand</small></summary>

In PyO3 0.15, the `#[pymethods]` attribute macro gained support for implementing "magic methods" such as `__str__` (aka "dunder" methods). This implementation was not quite finalized at the time, with a few edge cases to be decided upon. The existing `#[pyproto]` attribute macro was left untouched, because it covered these edge cases.

In PyO3 0.16, the `#[pymethods]` implementation has been completed and is now the preferred way to implement magic methods. To allow the PyO3 project to move forward, `#[pyproto]` has been deprecated (with expected removal in PyO3 0.18).

Migration from `#[pyproto]` to `#[pymethods]` is straightforward; copying the existing methods directly from the `#[pyproto]` trait implementation is all that is needed in most cases.

Before:

```rust,compile_fail
use pyo3::prelude::*;
use pyo3::class::{PyObjectProtocol, PyIterProtocol};
use pyo3::types::PyString;

#[pyclass]
struct MyClass {}

#[pyproto]
impl PyObjectProtocol for MyClass {
    fn __str__(&self) -> &'static [u8] {
        b"hello, world"
    }
}

#[pyproto]
impl PyIterProtocol for MyClass {
    fn __iter__(slf: PyRef<self>) -> PyResult<&PyAny> {
        PyString::new(slf.py(), "hello, world").iter()
    }
}
```

After

```rust,compile_fail
use pyo3::prelude::*;
use pyo3::types::PyString;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    fn __str__(&self) -> &'static [u8] {
        b"hello, world"
    }

    fn __iter__(slf: PyRef<self>) -> PyResult<&PyAny> {
        PyString::new(slf.py(), "hello, world").iter()
    }
}
```
</details>

### Removed `PartialEq` for object wrappers
<details>
<summary><small>Click to expand</small></summary>

The Python object wrappers `Py` and `PyAny` had implementations of `PartialEq`
so that `object_a == object_b` would compare the Python objects for pointer
equality, which corresponds to the `is` operator, not the `==` operator in
Python.  This has been removed in favor of a new method: use
`object_a.is(object_b)`.  This also has the advantage of not requiring the same
wrapper type for `object_a` and `object_b`; you can now directly compare a
`Py<T>` with a `&PyAny` without having to convert.

To check for Python object equality (the Python `==` operator), use the new
method `eq()`.
</details>

### Container magic methods now match Python behavior
<details>
<summary><small>Click to expand</small></summary>

In PyO3 0.15, `__getitem__`, `__setitem__` and `__delitem__` in `#[pymethods]` would generate only the _mapping_ implementation for a `#[pyclass]`. To match the Python behavior, these methods now generate both the _mapping_ **and** _sequence_ implementations.

This means that classes implementing these `#[pymethods]` will now also be treated as sequences, same as a Python `class` would be. Small differences in behavior may result:
 - PyO3 will allow instances of these classes to be cast to `PySequence` as well as `PyMapping`.
 - Python will provide a default implementation of `__iter__` (if the class did not have one) which repeatedly calls `__getitem__` with integers (starting at 0) until an `IndexError` is raised.

To explain this in detail, consider the following Python class:

```python
class ExampleContainer:

    def __len__(self):
        return 5

    def __getitem__(self, idx: int) -> int:
        if idx < 0 or idx > 5:
            raise IndexError()
        return idx
```

This class implements a Python [sequence](https://docs.python.org/3/glossary.html#term-sequence).

The `__len__` and `__getitem__` methods are also used to implement a Python [mapping](https://docs.python.org/3/glossary.html#term-mapping). In the Python C-API, these methods are not shared: the sequence `__len__` and `__getitem__` are defined by the `sq_length` and `sq_item` slots, and the mapping equivalents are `mp_length` and `mp_subscript`. There are similar distinctions for `__setitem__` and `__delitem__`.

Because there is no such distinction from Python, implementing these methods will fill the mapping and sequence slots simultaneously. A Python class with `__len__` implemented, for example, will have both the `sq_length` and `mp_length` slots filled.

The PyO3 behavior in 0.16 has been changed to be closer to this Python behavior by default.
</details>

### `wrap_pymodule!` and `wrap_pyfunction!` now respect privacy correctly
<details>
<summary><small>Click to expand</small></summary>

Prior to PyO3 0.16 the `wrap_pymodule!` and `wrap_pyfunction!` macros could use modules and functions whose defining `fn` was not reachable according Rust privacy rules.

For example, the following code was legal before 0.16, but in 0.16 is rejected because the `wrap_pymodule!` macro cannot access the `private_submodule` function:

```rust,compile_fail
mod foo {
    use pyo3::prelude::*;

    #[pymodule]
    fn private_submodule(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
        Ok(())
    }
}

use pyo3::prelude::*;
use foo::*;

#[pymodule]
fn my_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(private_submodule))?;
    Ok(())
}
```

To fix it, make the private submodule visible, e.g. with `pub` or `pub(crate)`.

```rust,ignore
mod foo {
    use pyo3::prelude::*;

    #[pymodule]
    pub(crate) fn private_submodule(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
        Ok(())
    }
}

use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use foo::*;

#[pymodule]
fn my_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(private_submodule))?;
    Ok(())
}
```
</details>

## from 0.14.* to 0.15

### Changes in sequence indexing
<details>
<summary><small>Click to expand</small></summary>

For all types that take sequence indices (`PyList`, `PyTuple` and `PySequence`),
the API has been made consistent to only take `usize` indices, for consistency
with Rust's indexing conventions.  Negative indices, which were only
sporadically supported even in APIs that took `isize`, now aren't supported
anywhere.

Further, the `get_item` methods now always return a `PyResult` instead of
panicking on invalid indices.  The `Index` trait has been implemented instead,
and provides the same panic behavior as on Rust vectors.

Note that *slice* indices (accepted by `PySequence::get_slice` and other) still
inherit the Python behavior of clamping the indices to the actual length, and
not panicking/returning an error on out of range indices.

An additional advantage of using Rust's indexing conventions for these types is
that these types can now also support Rust's indexing operators as part of a
consistent API:

```rust,ignore
#![allow(deprecated)]
use pyo3::{Python, types::PyList};

Python::with_gil(|py| {
    let list = PyList::new(py, &[1, 2, 3]);
    assert_eq!(list[0..2].to_string(), "[1, 2]");
});
```
</details>

## from 0.13.* to 0.14

### `auto-initialize` feature is now opt-in
<details>
<summary><small>Click to expand</small></summary>

For projects embedding Python in Rust, PyO3 no longer automatically initializes a Python interpreter on the first call to `Python::with_gil` (or `Python::acquire_gil`) unless the [`auto-initialize` feature](features.md#auto-initialize) is enabled.
</details>

### New `multiple-pymethods` feature
<details>
<summary><small>Click to expand</small></summary>

`#[pymethods]` have been reworked with a simpler default implementation which removes the dependency on the `inventory` crate. This reduces dependencies and compile times for the majority of users.

The limitation of the new default implementation is that it cannot support multiple `#[pymethods]` blocks for the same `#[pyclass]`. If you need this functionality, you must enable the `multiple-pymethods` feature which will switch `#[pymethods]` to the inventory-based implementation.
</details>

### Deprecated `#[pyproto]` methods
<details>
<summary><small>Click to expand</small></summary>

Some protocol (aka `__dunder__`) methods such as `__bytes__` and `__format__` have been possible to implement two ways in PyO3 for some time: via a `#[pyproto]` (e.g. `PyObjectProtocol` for the methods listed here), or by writing them directly in `#[pymethods]`. This is only true for a handful of the `#[pyproto]` methods (for technical reasons to do with the way PyO3 currently interacts with the Python C-API).

In the interest of having only one way to do things, the `#[pyproto]` forms of these methods have been deprecated.

To migrate just move the affected methods from a `#[pyproto]` to a `#[pymethods]` block.

Before:

```rust,compile_fail
use pyo3::prelude::*;
use pyo3::class::basic::PyObjectProtocol;

#[pyclass]
struct MyClass {}

#[pyproto]
impl PyObjectProtocol for MyClass {
    fn __bytes__(&self) -> &'static [u8] {
        b"hello, world"
    }
}
```

After:

```rust,no_run
use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    fn __bytes__(&self) -> &'static [u8] {
        b"hello, world"
    }
}
```
</details>

## from 0.12.* to 0.13

### Minimum Rust version increased to Rust 1.45
<details>
<summary><small>Click to expand</small></summary>

PyO3 `0.13` makes use of new Rust language features stabilized between Rust 1.40 and Rust 1.45. If you are using a Rust compiler older than Rust 1.45, you will need to update your toolchain to be able to continue using PyO3.
</details>

### Runtime changes to support the CPython limited API
<details>
<summary><small>Click to expand</small></summary>

In PyO3 `0.13` support was added for compiling against the CPython limited API. This had a number of implications for _all_ PyO3 users, described here.

The largest of these is that all types created from PyO3 are what CPython calls "heap" types. The specific implications of this are:

- If you wish to subclass one of these types _from Rust_ you must mark it `#[pyclass(subclass)]`, as you would if you wished to allow subclassing it from Python code.
- Type objects are now mutable - Python code can set attributes on them.
- `__module__` on types without `#[pyclass(module="mymodule")]` no longer returns `builtins`, it now raises `AttributeError`.
</details>

## from 0.11.* to 0.12

### `PyErr` has been reworked
<details>
<summary><small>Click to expand</small></summary>

In PyO3 `0.12` the `PyErr` type has been re-implemented to be significantly more compatible with
the standard Rust error handling ecosystem. Specifically `PyErr` now implements
`Error + Send + Sync`, which are the standard traits used for error types.

While this has necessitated the removal of a number of APIs, the resulting `PyErr` type should now
be much more easier to work with. The following sections list the changes in detail and how to
migrate to the new APIs.
</details>

#### `PyErr::new` and `PyErr::from_type` now require `Send + Sync` for their argument
<details>
<summary><small>Click to expand</small></summary>

For most uses no change will be needed. If you are trying to construct `PyErr` from a value that is
not `Send + Sync`, you will need to first create the Python object and then use
`PyErr::from_instance`.

Similarly, any types which implemented `PyErrArguments` will now need to be `Send + Sync`.
</details>

#### `PyErr`'s contents are now private
<details>
<summary><small>Click to expand</small></summary>

It is no longer possible to access the fields `.ptype`, `.pvalue` and `.ptraceback` of a `PyErr`.
You should instead now use the new methods `PyErr::ptype`, `PyErr::pvalue` and `PyErr::ptraceback`.
</details>

#### `PyErrValue` and `PyErr::from_value` have been removed
<details>
<summary><small>Click to expand</small></summary>

As these were part the internals of `PyErr` which have been reworked, these APIs no longer exist.

If you used this API, it is recommended to use `PyException::new_err` (see [the section on
Exception types](#exception-types-have-been-reworked)).
</details>

#### `Into<PyResult<T>>` for `PyErr` has been removed
<details>
<summary><small>Click to expand</small></summary>

This implementation was redundant. Just construct the `Result::Err` variant directly.

Before:
```rust,compile_fail
let result: PyResult<()> = PyErr::new::<TypeError, _>("error message").into();
```

After (also using the new reworked exception types; see the following section):
```rust,no_run
# use pyo3::{PyResult, exceptions::PyTypeError};
let result: PyResult<()> = Err(PyTypeError::new_err("error message"));
```
</details>

### Exception types have been reworked
<details>
<summary><small>Click to expand</small></summary>

Previously exception types were zero-sized marker types purely used to construct `PyErr`. In PyO3
0.12, these types have been replaced with full definitions and are usable in the same way as `PyAny`, `PyDict` etc. This
makes it possible to interact with Python exception objects.

The new types also have names starting with the "Py" prefix. For example, before:

```rust,ignore
let err: PyErr = TypeError::py_err("error message");
```

After:

```rust,ignore
# use pyo3::{PyErr, PyResult, Python, type_object::PyTypeObject};
# use pyo3::exceptions::{PyBaseException, PyTypeError};
# Python::with_gil(|py| -> PyResult<()> {
let err: PyErr = PyTypeError::new_err("error message");

// Uses Display for PyErr, new for PyO3 0.12
assert_eq!(err.to_string(), "TypeError: error message");

// Now possible to interact with exception instances, new for PyO3 0.12
let instance: &PyBaseException = err.instance(py);
assert_eq!(
    instance.getattr("__class__")?,
    PyTypeError::type_object(py).as_ref()
);
# Ok(())
# }).unwrap();
```
</details>

### `FromPy` has been removed
<details>
<summary><small>Click to expand</small></summary>

To simplify the PyO3 conversion traits, the `FromPy` trait has been removed. Previously there were
two ways to define the to-Python conversion for a type:
`FromPy<T> for PyObject` and `IntoPy<PyObject> for T`.

Now there is only one way to define the conversion, `IntoPy`, so downstream crates may need to
adjust accordingly.

Before:
```rust,compile_fail
# use pyo3::prelude::*;
struct MyPyObjectWrapper(PyObject);

impl FromPy<MyPyObjectWrapper> for PyObject {
    fn from_py(other: MyPyObjectWrapper, _py: Python<'_>) -> Self {
        other.0
    }
}
```

After
```rust,ignore
# use pyo3::prelude::*;
# #[allow(dead_code)]
struct MyPyObjectWrapper(PyObject);

# #[allow(deprecated)]
impl IntoPy<PyObject> for MyPyObjectWrapper {
    fn into_py(self, _py: Python<'_>) -> PyObject {
        self.0
    }
}
```

Similarly, code which was using the `FromPy` trait can be trivially rewritten to use `IntoPy`.

Before:
```rust,compile_fail
# use pyo3::prelude::*;
# Python::with_gil(|py| {
let obj = PyObject::from_py(1.234, py);
# })
```

After:
```rust,ignore
# #![allow(deprecated)]
# use pyo3::prelude::*;
# Python::with_gil(|py| {
let obj: PyObject = 1.234.into_py(py);
# })
```
</details>

### `PyObject` is now a type alias of `Py<PyAny>`
<details>
<summary><small>Click to expand</small></summary>

This should change very little from a usage perspective. If you implemented traits for both
`PyObject` and `Py<T>`, you may find you can just remove the `PyObject` implementation.
</details>

### `AsPyRef` has been removed
<details>
<summary><small>Click to expand</small></summary>

As `PyObject` has been changed to be just a type alias, the only remaining implementor of `AsPyRef`
was `Py<T>`. This removed the need for a trait, so the `AsPyRef::as_ref` method has been moved to
`Py::as_ref`.

This should require no code changes except removing `use pyo3::AsPyRef` for code which did not use
`pyo3::prelude::*`.

Before:
```rust,ignore
use pyo3::{AsPyRef, Py, types::PyList};
# pyo3::Python::with_gil(|py| {
let list_py: Py<PyList> = PyList::empty(py).into();
let list_ref: &PyList = list_py.as_ref(py);
# })
```

After:
```rust,ignore
use pyo3::{Py, types::PyList};
# pyo3::Python::with_gil(|py| {
let list_py: Py<PyList> = PyList::empty(py).into();
let list_ref: &PyList = list_py.as_ref(py);
# })
```
</details>

## from 0.10.* to 0.11

### Stable Rust
<details>
<summary><small>Click to expand</small></summary>

PyO3 now supports the stable Rust toolchain. The minimum required version is 1.39.0.
</details>

### `#[pyclass]` structs must now be `Send` or `unsendable`
<details>
<summary><small>Click to expand</small></summary>

Because `#[pyclass]` structs can be sent between threads by the Python interpreter, they must implement
`Send` or declared as `unsendable` (by `#[pyclass(unsendable)]`).
Note that `unsendable` is added in PyO3 `0.11.1` and `Send` is always required in PyO3 `0.11.0`.

This may "break" some code which previously was accepted, even though it could be unsound.
There can be two fixes:

1. If you think that your `#[pyclass]` actually must be `Send`able, then let's implement `Send`.
   A common, safer way is using thread-safe types. E.g., `Arc` instead of `Rc`, `Mutex` instead of
   `RefCell`, and `Box<dyn Send + T>` instead of `Box<dyn T>`.

   Before:
   ```rust,compile_fail
   use pyo3::prelude::*;
   use std::rc::Rc;
   use std::cell::RefCell;

   #[pyclass]
   struct NotThreadSafe {
       shared_bools: Rc<RefCell<Vec<bool>>>,
       closure: Box<dyn Fn()>,
   }
   ```

   After:
   ```rust,ignore
   # #![allow(dead_code)]
   use pyo3::prelude::*;
   use std::sync::{Arc, Mutex};

   #[pyclass]
   struct ThreadSafe {
       shared_bools: Arc<Mutex<Vec<bool>>>,
       closure: Box<dyn Fn() + Send>,
   }
   ```

   In situations where you cannot change your `#[pyclass]` to automatically implement `Send`
   (e.g., when it contains a raw pointer), you can use `unsafe impl Send`.
   In such cases, care should be taken to ensure the struct is actually thread safe.
   See [the Rustonomicon](https://doc.rust-lang.org/nomicon/send-and-sync.html) for more.

2. If you think that your `#[pyclass]` should not be accessed by another thread, you can use
   `unsendable` flag. A class marked with `unsendable` panics when accessed by another thread,
   making it thread-safe to expose an unsendable object to the Python interpreter.

   Before:
   ```rust,compile_fail
   use pyo3::prelude::*;

   #[pyclass]
   struct Unsendable {
       pointers: Vec<*mut std::ffi::c_char>,
   }
   ```

   After:
   ```rust,no_run
   # #![allow(dead_code)]
   use pyo3::prelude::*;

   #[pyclass(unsendable)]
   struct Unsendable {
       pointers: Vec<*mut std::ffi::c_char>,
   }
   ```
</details>

### All `PyObject` and `Py<T>` methods now take `Python` as an argument
<details>
<summary><small>Click to expand</small></summary>

Previously, a few methods such as `Object::get_refcnt` did not take `Python` as an argument (to
ensure that the Python GIL was held by the current thread). Technically, this was not sound.
To migrate, just pass a `py` argument to any calls to these methods.

Before:
```rust,compile_fail
# pyo3::Python::attach(|py| {
py.None().get_refcnt();
# })
```

After:
```rust
# pyo3::Python::attach(|py| {
py.None().get_refcnt(py);
# })
```
</details>

## from 0.9.* to 0.10

### `ObjectProtocol` is removed
<details>
<summary><small>Click to expand</small></summary>

All methods are moved to [`PyAny`].
And since now all native types (e.g., `PyList`) implements `Deref<Target=PyAny>`,
all you need to do is remove `ObjectProtocol` from your code.
Or if you use `ObjectProtocol` by `use pyo3::prelude::*`, you have to do nothing.

Before:
```rust,compile_fail,ignore
use pyo3::ObjectProtocol;

# pyo3::Python::with_gil(|py| {
let obj = py.eval("lambda: 'Hi :)'", None, None).unwrap();
let hi: &pyo3::types::PyString = obj.call0().unwrap().downcast().unwrap();
assert_eq!(hi.len().unwrap(), 5);
# })
```

After:
```rust,ignore
# pyo3::Python::with_gil(|py| {
let obj = py.eval("lambda: 'Hi :)'", None, None).unwrap();
let hi: &pyo3::types::PyString = obj.call0().unwrap().downcast().unwrap();
assert_eq!(hi.len().unwrap(), 5);
# })
```
</details>

### No `#![feature(specialization)]` in user code
<details>
<summary><small>Click to expand</small></summary>

While PyO3 itself still requires specialization and nightly Rust,
now you don't have to use `#![feature(specialization)]` in your crate.
</details>

## from 0.8.* to 0.9

### `#[new]` interface
<details>
<summary><small>Click to expand</small></summary>

[`PyRawObject`](https://docs.rs/pyo3/0.8.5/pyo3/type_object/struct.PyRawObject.html)
is now removed and our syntax for constructors has changed.

Before:
```rust,compile_fail
#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[new]
    fn new(obj: &PyRawObject) {
        obj.init(MyClass {})
    }
}
```

After:
```rust,no_run
# use pyo3::prelude::*;
#[pyclass]
struct MyClass {}

#[pymethods]
impl MyClass {
    #[new]
    fn new() -> Self {
        MyClass {}
    }
}
```

Basically you can return `Self` or `Result<Self>` directly.
For more, see [the constructor section](class.md#constructor) of this guide.
</details>

### PyCell
<details>
<summary><small>Click to expand</small></summary>

PyO3 0.9 introduces `PyCell`, which is a [`RefCell`]-like object wrapper
for ensuring Rust's rules regarding aliasing of references are upheld.
For more detail, see the
[Rust Book's section on Rust's rules of references](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#the-rules-of-references)

For `#[pymethods]` or `#[pyfunction]`s, your existing code should continue to work without any change.
Python exceptions will automatically be raised when your functions are used in a way which breaks Rust's
rules of references.

Here is an example.
```rust
# use pyo3::prelude::*;

#[pyclass]
struct Names {
    names: Vec<String>,
}

#[pymethods]
impl Names {
    #[new]
    fn new() -> Self {
        Names { names: vec![] }
    }
    fn merge(&mut self, other: &mut Names) {
        self.names.append(&mut other.names)
    }
}
# Python::attach(|py| {
#     let names = Py::new(py, Names::new()).unwrap();
#     pyo3::py_run!(py, names, r"
#     try:
#        names.merge(names)
#        assert False, 'Unreachable'
#     except RuntimeError as e:
#        assert str(e) == 'Already borrowed'
#     ");
# })
```
`Names` has a `merge` method, which takes `&mut self` and another argument of type `&mut Self`.
Given this `#[pyclass]`, calling `names.merge(names)` in Python raises
a [`PyBorrowMutError`] exception, since it requires two mutable borrows of `names`.

However, for `#[pyproto]` and some functions, you need to manually fix the code.

#### Object creation
In 0.8 object creation was done with `PyRef::new` and `PyRefMut::new`.
In 0.9 these have both been removed.
To upgrade code, please use
`PyCell::new` instead.
If you need [`PyRef`] or [`PyRefMut`], just call `.borrow()` or `.borrow_mut()`
on the newly-created `PyCell`.

Before:
```rust,compile_fail
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {}
# Python::with_gil(|py| {
let obj_ref = PyRef::new(py, MyClass {}).unwrap();
# })
```

After:
```rust,ignore
# use pyo3::prelude::*;
# #[pyclass]
# struct MyClass {}
# Python::with_gil(|py| {
let obj = PyCell::new(py, MyClass {}).unwrap();
let obj_ref = obj.borrow();
# })
```

#### Object extraction
For `PyClass` types `T`, `&T` and `&mut T` no longer have [`FromPyObject`] implementations.
Instead you should extract `PyRef<T>` or `PyRefMut<T>`, respectively.
If `T` implements `Clone`, you can extract `T` itself.
In addition, you can also extract `&PyCell<T>`, though you rarely need it.

Before:
```compile_fail
let obj: &PyAny = create_obj();
let obj_ref: &MyClass = obj.extract().unwrap();
let obj_ref_mut: &mut MyClass = obj.extract().unwrap();
```

After:
```rust,ignore
# use pyo3::prelude::*;
# use pyo3::types::IntoPyDict;
# #[pyclass] #[derive(Clone)] struct MyClass {}
# #[pymethods] impl MyClass { #[new]fn new() -> Self { MyClass {} }}
# Python::with_gil(|py| {
# let typeobj = py.get_type::<MyClass>();
# let d = [("c", typeobj)].into_py_dict(py);
# let create_obj = || py.eval("c()", None, Some(d)).unwrap();
let obj: &PyAny = create_obj();
let obj_cell: &PyCell<MyClass> = obj.extract().unwrap();
let obj_cloned: MyClass = obj.extract().unwrap(); // extracted by cloning the object
{
    let obj_ref: PyRef<'_, MyClass> = obj.extract().unwrap();
    // we need to drop obj_ref before we can extract a PyRefMut due to Rust's rules of references
}
let obj_ref_mut: PyRefMut<'_, MyClass> = obj.extract().unwrap();
# })
```


#### `#[pyproto]`
Most of the arguments to methods in `#[pyproto]` impls require a
[`FromPyObject`] implementation.
So if your protocol methods take `&T` or `&mut T` (where `T: PyClass`),
please use [`PyRef`] or [`PyRefMut`] instead.

Before:
```rust,compile_fail
# use pyo3::prelude::*;
# use pyo3::class::PySequenceProtocol;
#[pyclass]
struct ByteSequence {
    elements: Vec<u8>,
}
#[pyproto]
impl PySequenceProtocol for ByteSequence {
    fn __concat__(&self, other: &Self) -> PyResult<Self> {
        let mut elements = self.elements.clone();
        elements.extend_from_slice(&other.elements);
        Ok(Self { elements })
    }
}
```

After:
```rust,compile_fail
# use pyo3::prelude::*;
# use pyo3::class::PySequenceProtocol;
#[pyclass]
struct ByteSequence {
    elements: Vec<u8>,
}
#[pyproto]
impl PySequenceProtocol for ByteSequence {
    fn __concat__(&self, other: PyRef<'p, Self>) -> PyResult<Self> {
        let mut elements = self.elements.clone();
        elements.extend_from_slice(&other.elements);
        Ok(Self { elements })
    }
}
```
</details>

<style>
    /* render details immediately below h3 headers */
    h3:has(+ details) {
        margin-bottom: 0;
    }

    /* make summary text hint that it's clickable and increase the
       size of the clickable area by padding downwards */
    details > summary {
        cursor: pointer;
        padding-bottom: 0.5em;
    }

    /* reduce margin from paragraph directly below the clickable space
       to avoid large gap */
    details > summary + p {
        margin-block-start: 0.5em;
    }

    /* pack headings that aren't expanded slightly closer together */
    h3 + details:not([open]) + h3 {
        margin-top: 1.5em;
    }
</style>

[`FromPyObject`]: {{#PYO3_DOCS_URL}}/pyo3/conversion/trait.FromPyObject.html
[`PyAny`]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyAny.html
[`PyBorrowMutError`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyBorrowMutError.html
[`PyRef`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyRef.html
[`PyRefMut`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyRef.html

[`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
