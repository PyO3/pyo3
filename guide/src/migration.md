# Migrating from older PyO3 versions

This guide can help you upgrade code through breaking changes from one PyO3 version to the next.
For a detailed list of all changes, see the [CHANGELOG](changelog.md).

## from 0.20.* to 0.21

PyO3 0.21 introduces a new `Bound<'py, T>` smart pointer which replaces the existing "GIL Refs" API to interact with Python objects. For example, in PyO3 0.20 the reference `&'py PyAny` would be used to interact with Python objects. In PyO3 0.21 the updated type is `Bound<'py, PyAny>`. Making this change moves Rust ownership semantics out of PyO3's internals and into user code. This change fixes [a known soundness edge case of interaction with gevent](https://github.com/PyO3/pyo3/issues/3668) as well as improves CPU and [memory performance](https://github.com/PyO3/pyo3/issues/1056). For a full history of discussion see https://github.com/PyO3/pyo3/issues/3382.

The "GIL Ref" `&'py PyAny` and similar types such as `&'py PyDict` continue to be available as a deprecated API. Due to the advantages of the new API it is advised that all users make the effort to upgrade as soon as possible.

In addition to the major API type overhaul, PyO3 has needed to make a few small breaking adjustments to other APIs to close correctness and soundness gaps.

The recommended steps to update to PyO3 0.21 is as follows:
  1. Enable the `gil-refs` feature to silence deprecations related to the API change
  2. Fix all other PyO3 0.21 migration steps
  3. Disable the `gil-refs` feature and migrate off the deprecated APIs

The following sections are laid out in this order.

### Enable the `gil-refs` feature

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

### `PyTypeInfo` and `PyTryFrom` have been adjusted

The `PyTryFrom` trait has aged poorly, its [`try_from`] method now conflicts with `try_from` in the 2021 edition prelude. A lot of its functionality was also duplicated with `PyTypeInfo`.

To tighten up the PyO3 traits as part of the deprecation of the GIL Refs API the `PyTypeInfo` trait has had a simpler companion `PyTypeCheck`. The methods [`PyAny::downcast`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyAny.html#method.downcast) and [`PyAny::downcast_exact`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyAny.html#method.downcast_exact) no longer use `PyTryFrom` as a bound, instead using `PyTypeCheck` and `PyTypeInfo` respectively.

To migrate, switch all type casts to use `obj.downcast()` instead of `try_from(obj)` (and similar for `downcast_exact`).

Before:

```rust
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

```rust
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

### `Iter(A)NextOutput` are deprecated

The `__next__` and `__anext__` magic methods can now return any type convertible into Python objects directly just like all other `#[pymethods]`. The `IterNextOutput` used by `__next__` and `IterANextOutput` used by `__anext__` are subsequently deprecated. Most importantly, this change allows returning an awaitable from `__anext__` without non-sensically wrapping it into `Yield` or `Some`. Only the return types `Option<T>` and `Result<Option<T>, E>` are still handled in a special manner where `Some(val)` yields `val` and `None` stops iteration.

Starting with an implementation of a Python iterator using `IterNextOutput`, e.g.

```rust
#![allow(deprecated)]
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

```rust
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

```rust
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

```rust
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
        PyClassAwaitable { number: self.number }
    }

    fn __aiter__(slf: Py<Self>) -> Py<Self> {
        slf
    }
}
```

### `PyType::name` has been renamed to `PyType::qualname`

`PyType::name` has been renamed to `PyType::qualname` to indicate that it does indeed return the [qualified name](https://docs.python.org/3/glossary.html#term-qualified-name), matching the `__qualname__` attribute. The newly added `PyType::name` yields the full name including the module name now which corresponds to `__module__.__name__` on the level of attributes.

### `PyCell` has been deprecated

Interactions with Python objects implemented in Rust no longer need to go though `PyCell<T>`. Instead iteractions with Python object now consistently go through `Bound<T>` or `Py<T>` independently of whether `T` is native Python object or a `#[pyclass]` implemented in Rust. Use `Bound::new` or `Py::new` respectively to create and `Bound::borrow(_mut)` / `Py::borrow(_mut)` to borrow the Rust object.

### Migrating from the GIL-Refs API to `Bound<T>`

To minimise breakage of code using the GIL-Refs API, the `Bound<T>` smart pointer has been introduced by adding complements to all functions which accept or return GIL Refs. This allows code to migrate by replacing the deprecated APIs with the new ones.

To identify what to migrate, temporarily switch off the `gil-refs` feature to see deprecation warnings on all uses of APIs accepting and producing GIL Refs. Over one or more PRs it should be possible to follow the deprecation hints to update code. Depending on your development environment, switching off the `gil-refs` feature may introduce [some very targeted breakages](#deactivating-the-gil-refs-feature), so you may need to fixup those first.

For example, the following APIs have gained updated variants:
- `PyList::new`, `PyTyple::new` and similar constructors have replacements `PyList::new_bound`, `PyTuple::new_bound` etc.
- `FromPyObject::extract` has a new `FromPyObject::extract_bound` (see the section below)
- The `PyTypeInfo` trait has had new `_bound` methods added to accept / return `Bound<T>`.

Because the new `Bound<T>` API brings ownership out of the PyO3 framework and into user code, there are a few places where user code is expected to need to adjust while switching to the new API:
- Code will need to add the occasional `&` to borrow the new smart pointer as `&Bound<T>` to pass these types around (or use `.clone()` at the very small cost of increasing the Python reference count)
- `Bound<PyList>` and `Bound<PyTuple>` cannot support indexing with `list[0]`, you should use `list.get_item(0)` instead.
- `Bound<PyTuple>::iter_borrowed` is slightly more efficient than `Bound<PyTuple>::iter`. The default iteration of `Bound<PyTuple>` cannot return borrowed references because Rust does not (yet) have "lending iterators". Similarly `Bound<PyTuple>::get_borrowed_item` is more efficient than `Bound<PyTuple>::get_item` for the same reason.
- `&Bound<T>` does not implement `FromPyObject` (although it might be possible to do this in the future once the GIL Refs API is completely removed). Use `bound_any.downcast::<T>()` instead of `bound_any.extract::<&Bound<T>>()`.
- To convert between `&PyAny` and `&Bound<PyAny>` you can use the `as_borrowed()` method:

```rust,ignore
let gil_ref: &PyAny = ...;
let bound: &Bound<PyAny> = &gil_ref.as_borrowed();
```

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

### Deactivating the `gil-refs` feature

As a final step of migration, deactivating the `gil-refs` feature will set up code for best performance and is intended to set up a forward-compatible API for PyO3 0.22.

There is one notable API removed when this feature is disabled. `FromPyObject` trait implementations for types which borrow directly from the input data cannot be implemented by PyO3 without GIL Refs (while the migration is ongoing). These types are `&str`, `Cow<'_, str>`, `&[u8]`, `Cow<'_, u8>`.

To ease pain during migration, these types instead implement a new temporary trait `FromPyObjectBound` which is the expected future form of `FromPyObject`. The new temporary trait ensures is that `obj.extract::<&str>()` continues to work (with the new constraint that the extracted value now depends on the input `obj` lifetime), as well for these types in `#[pyfunction]` arguments.

An unfortunate final point here is that PyO3 cannot offer this new implementation for `&str` on `abi3` builds older than Python 3.10. On code which needs to build for 3.10 or older, many cases of `.extract::<&str>()` may need to be replaced with `.extract::<PyBackedStr>()`, which is string data which borrows from the Python `str` object. Alternatively, use `.extract::<Cow<str>>()`, `.extract::<String>()` to copy the data into Rust for these versions.

## from 0.19.* to 0.20

### Drop support for older technologies

PyO3 0.20 has increased minimum Rust version to 1.56. This enables use of newer language features and simplifies maintenance of the project.

### `PyDict::get_item` now returns a `Result`

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
    assert!(dict.get_item_with_error(dict).unwrap_err().is_instance_of::<PyTypeError>(py));
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
    assert!(dict.get_item(dict).unwrap_err().is_instance_of::<PyTypeError>(py));

    Ok(())
});
# }
```

### Required arguments are no longer accepted after optional arguments

[Trailing `Option<T>` arguments](./function/signature.md#trailing-optional-arguments) have an automatic default of `None`. To avoid unwanted changes when modifying function signatures, in PyO3 0.18 it was deprecated to have a required argument after an `Option<T>` argument without using `#[pyo3(signature = (...))]` to specify the intended defaults. In PyO3 0.20, this becomes a hard error.

Before:

```rust,ignore
#[pyfunction]
fn x_or_y(x: Option<u64>, y: u64) -> u64 {
    x.unwrap_or(y)
}
```

After:

```rust
# #![allow(dead_code)]
# use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (x, y))] // both x and y have no defaults and are required
fn x_or_y(x: Option<u64>, y: u64) -> u64 {
    x.unwrap_or(y)
}
```

### Remove deprecated function forms

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

```rust
# #![allow(dead_code)]
# use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (a, b=0, /))]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
```

### `IntoPyPointer` trait removed

The trait `IntoPyPointer`, which provided the `into_ptr` method on many types, has been removed. `into_ptr` is now available as an inherent method on all types that previously implemented this trait.

### `AsPyPointer` now `unsafe` trait

The trait `AsPyPointer` is now `unsafe trait`, meaning any external implementation of it must be marked as `unsafe impl`, and ensure that they uphold the invariant of returning valid pointers.

## from 0.18.* to 0.19

### Access to `Python` inside `__traverse__` implementations are now forbidden

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

### Smarter `anyhow::Error` / `eyre::Report` conversion when inner error is "simple" `PyErr`

When converting from `anyhow::Error` or `eyre::Report` to `PyErr`, if the inner error is a "simple" `PyErr` (with no source error), then the inner error will be used directly as the `PyErr` instead of wrapping it in a new `PyRuntimeError` with the original information converted into a string.

```rust
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

### The deprecated `Python::acquire_gil` was removed and `Python::with_gil` must be used instead

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

The replacement is [`Python::with_gil`]() which is more cumbersome but enforces the proper nesting by design, e.g.

```rust
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

// It either forces us to release the GIL before aquiring it again.
let first = Python::with_gil(|py| Object::new(py));
let second = Python::with_gil(|py| Object::new(py));
drop(first);
drop(second);

// Or it ensure releasing the inner lock before the outer one.
Python::with_gil(|py| {
    let first = Object::new(py);
    let second = Python::with_gil(|py| Object::new(py));
    drop(first);
    drop(second);
});
```

Furthermore, `Python::acquire_gil` provides ownership of a `GILGuard` which can be freely stored and passed around. This is usually not helpful as it may keep the lock held for a long time thereby blocking progress in other parts of the program. Due to the generative lifetime attached to the GIL token supplied by `Python::with_gil`, the problem is avoided as the GIL token can only be passed down the call chain. Often, this issue can also be avoided entirely as any GIL-bound reference `&'py PyAny` implies access to a GIL token `Python<'py>` via the [`PyAny::py`](https://docs.rs/pyo3/latest/pyo3/types/struct.PyAny.html#method.py) method.

## from 0.17.* to 0.18

### Required arguments after `Option<_>` arguments will no longer be automatically inferred

In `#[pyfunction]` and `#[pymethods]`, if a "required" function input such as `i32` came after an `Option<_>` input, then the `Option<_>` would be implicitly treated as required. (All trailing `Option<_>` arguments were treated as optional with a default value of `None`).

Starting with PyO3 0.18, this is deprecated and a future PyO3 version will require a [`#[pyo3(signature = (...))]` option](./function/signature.md) to explicitly declare the programmer's intention.

Before, x in the below example would be required to be passed from Python code:

```rust,compile_fail
# #![allow(dead_code)]
# use pyo3::prelude::*;

#[pyfunction]
fn required_argument_after_option(x: Option<i32>, y: i32) {}
```

After, specify the intended Python signature explicitly:

```rust
# #![allow(dead_code)]
# use pyo3::prelude::*;

// If x really was intended to be required
#[pyfunction(signature = (x, y))]
fn required_argument_after_option_a(x: Option<i32>, y: i32) {}

// If x was intended to be optional, y needs a default too
#[pyfunction(signature = (x=None, y=0))]
fn required_argument_after_option_b(x: Option<i32>, y: i32) {}
```

### `__text_signature__` is now automatically generated for `#[pyfunction]` and `#[pymethods]`

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
#     Python::with_gil(|py| {
#         let simple = wrap_pyfunction!(simple_function, py).unwrap();
#         assert_eq!(simple.getattr("__text_signature__").unwrap().to_string(), "(a, b, c)");
#         let defaulted = wrap_pyfunction!(function_with_defaults, py).unwrap();
#         assert_eq!(defaulted.getattr("__text_signature__").unwrap().to_string(), "(a, b=1, c=2)");
#     })
# }
```

## from 0.16.* to 0.17

### Type checks have been changed for `PyMapping` and `PySequence` types

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

### The `multiple-pymethods` feature now requires Rust 1.62

Due to limitations in the `inventory` crate which the `multiple-pymethods` feature depends on, this feature now
requires Rust 1.62. For more information see [dtolnay/inventory#32](https://github.com/dtolnay/inventory/issues/32).

### Added `impl IntoPy<Py<PyString>> for &str`

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

```rust
# use pyo3::prelude::*;
#
# fn main() {
Python::with_gil(|py| {
    let _test: Py<PyAny> = "test".into_py(py);
});
# }
```

### The `pyproto` feature is now disabled by default

In preparation for removing the deprecated `#[pyproto]` attribute macro in a future PyO3 version, it is now gated behind an opt-in feature flag. This also gives a slight saving to compile times for code which does not use the deprecated macro.

### `PyTypeObject` trait has been deprecated

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

### `impl<T, const N: usize> IntoPy<PyObject> for [T; N]` now requires `T: IntoPy` rather than `T: ToPyObject`

If this leads to errors, simply implement `IntoPy`. Because pyclasses already implement `IntoPy`, you probably don't need to worry about this.

### Each `#[pymodule]` can now only be initialized once per process

To make PyO3 modules sound in the presence of Python sub-interpreters, for now it has been necessary to explicitly disable the ability to initialize a `#[pymodule]` more than once in the same process. Attempting to do this will now raise an `ImportError`.

## from 0.15.* to 0.16

### Drop support for older technologies

PyO3 0.16 has increased minimum Rust version to 1.48 and minimum Python version to 3.7. This enables use of newer language features (enabling some of the other additions in 0.16) and simplifies maintenance of the project.

### `#[pyproto]` has been deprecated

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

### Removed `PartialEq` for object wrappers

The Python object wrappers `Py` and `PyAny` had implementations of `PartialEq`
so that `object_a == object_b` would compare the Python objects for pointer
equality, which corresponds to the `is` operator, not the `==` operator in
Python.  This has been removed in favor of a new method: use
`object_a.is(object_b)`.  This also has the advantage of not requiring the same
wrapper type for `object_a` and `object_b`; you can now directly compare a
`Py<T>` with a `&PyAny` without having to convert.

To check for Python object equality (the Python `==` operator), use the new
method `eq()`.

### Container magic methods now match Python behavior

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

### `wrap_pymodule!` and `wrap_pyfunction!` now respect privacy correctly

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

```rust
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

## from 0.14.* to 0.15

### Changes in sequence indexing

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

```rust
#![allow(deprecated)]
use pyo3::{Python, types::PyList};

Python::with_gil(|py| {
    let list = PyList::new(py, &[1, 2, 3]);
    assert_eq!(list[0..2].to_string(), "[1, 2]");
});
```

## from 0.13.* to 0.14

### `auto-initialize` feature is now opt-in

For projects embedding Python in Rust, PyO3 no longer automatically initializes a Python interpreter on the first call to `Python::with_gil` (or `Python::acquire_gil`) unless the [`auto-initialize` feature](features.md#auto-initialize) is enabled.

### New `multiple-pymethods` feature

`#[pymethods]` have been reworked with a simpler default implementation which removes the dependency on the `inventory` crate. This reduces dependencies and compile times for the majority of users.

The limitation of the new default implementation is that it cannot support multiple `#[pymethods]` blocks for the same `#[pyclass]`. If you need this functionality, you must enable the `multiple-pymethods` feature which will switch `#[pymethods]` to the inventory-based implementation.

### Deprecated `#[pyproto]` methods

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

```rust
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

## from 0.12.* to 0.13

### Minimum Rust version increased to Rust 1.45

PyO3 `0.13` makes use of new Rust language features stabilized between Rust 1.40 and Rust 1.45. If you are using a Rust compiler older than Rust 1.45, you will need to update your toolchain to be able to continue using PyO3.

### Runtime changes to support the CPython limited API

In PyO3 `0.13` support was added for compiling against the CPython limited API. This had a number of implications for _all_ PyO3 users, described here.

The largest of these is that all types created from PyO3 are what CPython calls "heap" types. The specific implications of this are:

- If you wish to subclass one of these types _from Rust_ you must mark it `#[pyclass(subclass)]`, as you would if you wished to allow subclassing it from Python code.
- Type objects are now mutable - Python code can set attributes on them.
- `__module__` on types without `#[pyclass(module="mymodule")]` no longer returns `builtins`, it now raises `AttributeError`.

## from 0.11.* to 0.12

### `PyErr` has been reworked

In PyO3 `0.12` the `PyErr` type has been re-implemented to be significantly more compatible with
the standard Rust error handling ecosystem. Specifically `PyErr` now implements
`Error + Send + Sync`, which are the standard traits used for error types.

While this has necessitated the removal of a number of APIs, the resulting `PyErr` type should now
be much more easier to work with. The following sections list the changes in detail and how to
migrate to the new APIs.

#### `PyErr::new` and `PyErr::from_type` now require `Send + Sync` for their argument

For most uses no change will be needed. If you are trying to construct `PyErr` from a value that is
not `Send + Sync`, you will need to first create the Python object and then use
`PyErr::from_instance`.

Similarly, any types which implemented `PyErrArguments` will now need to be `Send + Sync`.

#### `PyErr`'s contents are now private

It is no longer possible to access the fields `.ptype`, `.pvalue` and `.ptraceback` of a `PyErr`.
You should instead now use the new methods `PyErr::ptype`, `PyErr::pvalue` and `PyErr::ptraceback`.

#### `PyErrValue` and `PyErr::from_value` have been removed

As these were part the internals of `PyErr` which have been reworked, these APIs no longer exist.

If you used this API, it is recommended to use `PyException::new_err` (see [the section on
Exception types](#exception-types-have-been-reworked)).

#### `Into<PyResult<T>>` for `PyErr` has been removed

This implementation was redundant. Just construct the `Result::Err` variant directly.

Before:
```rust,compile_fail
let result: PyResult<()> = PyErr::new::<TypeError, _>("error message").into();
```

After (also using the new reworked exception types; see the following section):
```rust
# use pyo3::{PyResult, exceptions::PyTypeError};
let result: PyResult<()> = Err(PyTypeError::new_err("error message"));
```

### Exception types have been reworked

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

### `FromPy` has been removed
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
```rust
# use pyo3::prelude::*;
struct MyPyObjectWrapper(PyObject);

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
```rust
# use pyo3::prelude::*;
# Python::with_gil(|py| {
let obj: PyObject = 1.234.into_py(py);
# })
```

### `PyObject` is now a type alias of `Py<PyAny>`
This should change very little from a usage perspective. If you implemented traits for both
`PyObject` and `Py<T>`, you may find you can just remove the `PyObject` implementation.

### `AsPyRef` has been removed
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

## from 0.10.* to 0.11

### Stable Rust
PyO3 now supports the stable Rust toolchain. The minimum required version is 1.39.0.

### `#[pyclass]` structs must now be `Send` or `unsendable`
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
   ```rust
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
       pointers: Vec<*mut std::os::raw::c_char>,
   }
   ```

   After:
   ```rust
   # #![allow(dead_code)]
   use pyo3::prelude::*;

   #[pyclass(unsendable)]
   struct Unsendable {
       pointers: Vec<*mut std::os::raw::c_char>,
   }
   ```

### All `PyObject` and `Py<T>` methods now take `Python` as an argument
Previously, a few methods such as `Object::get_refcnt` did not take `Python` as an argument (to
ensure that the Python GIL was held by the current thread). Technically, this was not sound.
To migrate, just pass a `py` argument to any calls to these methods.

Before:
```rust,compile_fail
# pyo3::Python::with_gil(|py| {
py.None().get_refcnt();
# })
```

After:
```rust
# pyo3::Python::with_gil(|py| {
py.None().get_refcnt(py);
# })
```

## from 0.9.* to 0.10

### `ObjectProtocol` is removed
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

### No `#![feature(specialization)]` in user code
While PyO3 itself still requires specialization and nightly Rust,
now you don't have to use `#![feature(specialization)]` in your crate.

## from 0.8.* to 0.9

### `#[new]` interface
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
```rust
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
For more, see [the constructor section](class.html#constructor) of this guide.

### PyCell
PyO3 0.9 introduces [`PyCell`], which is a [`RefCell`]-like object wrapper
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
# Python::with_gil(|py| {
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
[`PyCell::new`]({{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyCell.html#method.new) instead.
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

[`FromPyObject`]: {{#PYO3_DOCS_URL}}/pyo3/conversion/trait.FromPyObject.html
[`PyAny`]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyAny.html
[`PyCell`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyCell.html
[`PyBorrowMutError`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyBorrowMutError.html
[`PyRef`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyRef.html
[`PyRefMut`]: {{#PYO3_DOCS_URL}}/pyo3/pycell/struct.PyRef.html

[`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
