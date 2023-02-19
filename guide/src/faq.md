# Frequently Asked Questions and troubleshooting

## I'm experiencing deadlocks using PyO3 with lazy_static or once_cell!

`lazy_static` and `once_cell::sync` both use locks to ensure that initialization is performed only by a single thread. Because the Python GIL is an additional lock this can lead to deadlocks in the following way:

1. A thread (thread A) which has acquired the Python GIL starts initialization of a `lazy_static` value.
2. The initialization code calls some Python API which temporarily releases the GIL e.g. `Python::import`.
3. Another thread (thread B) acquires the Python GIL and attempts to access the same `lazy_static` value.
4. Thread B is blocked, because it waits for `lazy_static`'s initialization to lock to release.
5. Thread A is blocked, because it waits to re-acquire the GIL which thread B still holds.
6. Deadlock.

PyO3 provides a struct [`GILOnceCell`] which works equivalently to `OnceCell` but relies solely on the Python GIL for thread safety. This means it can be used in place of `lazy_static` or `once_cell` where you are experiencing the deadlock described above. See the documentation for [`GILOnceCell`] for an example how to use it.

[`GILOnceCell`]: {{#PYO3_DOCS_URL}}/pyo3/once_cell/struct.GILOnceCell.html

## I can't run `cargo test`; or I can't build in a Cargo workspace: I'm having linker issues like "Symbol not found" or "Undefined reference to _PyExc_SystemError"!

Currently, [#340](https://github.com/PyO3/pyo3/issues/340) causes `cargo test` to fail with linking errors when the `native-module` feature is activated. Linking errors can also happen when building in a cargo workspace where a different crate also uses PyO3 (see [#2521](https://github.com/PyO3/pyo3/issues/2521)). For now, there are three ways we can work around these issues.

1. Make the `native-module` feature optional. Build with `maturin develop --features native`

```toml
[dependencies.pyo3]
{{#PYO3_CRATE_VERSION}}

[features]
native-module = ["pyo3/native-module"]
```

2. Make the `native-module` feature optional and default. Run tests with `cargo test --no-default-features`:

```toml
[dependencies.pyo3]
{{#PYO3_CRATE_VERSION}}

[features]
native-module = ["pyo3/native-module"]
default = ["native-module"]
```

3. If you are using a [`pyproject.toml`](https://maturin.rs/metadata.html) file to control maturin settings, add the following section:

```toml
[tool.maturin]
features = ["pyo3/native-module"]
# Or for maturin 0.12:
# cargo-extra-args = ["--features", "pyo3/native-module"]
```

## I can't run `cargo test`: my crate cannot be found for tests in `tests/` directory!

The Rust book suggests to [put integration tests inside a `tests/` directory](https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests).

For a PyO3 `native-module` project where the `crate-type` is set to `"cdylib"` in your `Cargo.toml`,
the compiler won't be able to find your crate and will display errors such as `E0432` or `E0463`:

```text
error[E0432]: unresolved import `my_crate`
 --> tests/test_my_crate.rs:1:5
  |
1 | use my_crate;
  |     ^^^^^^^^^^^^ no external crate `my_crate`
```

The best solution is to make your crate types include both `rlib` and `cdylib`:

```toml
# Cargo.toml
[lib]
crate-type = ["cdylib", "rlib"]
```

## Ctrl-C doesn't do anything while my Rust code is executing!

This is because Ctrl-C raises a SIGINT signal, which is handled by the calling Python process by simply setting a flag to action upon later. This flag isn't checked while Rust code called from Python is executing, only once control returns to the Python interpreter.

You can give the Python interpreter a chance to process the signal properly by calling `Python::check_signals`. It's good practice to call this function regularly if you have a long-running Rust function so that your users can cancel it.

## `#[pyo3(get)]` clones my field!

You may have a nested struct similar to this:

```rust
# use pyo3::prelude::*;
#[pyclass]
#[derive(Clone)]
struct Inner {/* fields omitted */}

#[pyclass]
struct Outer {
    #[pyo3(get)]
    inner: Inner,
}

#[pymethods]
impl Outer {
    #[new]
    fn __new__() -> Self {
        Self { inner: Inner {} }
    }
}
```

When Python code accesses `Outer`'s field, PyO3 will return a new object on every access (note that their addresses are different):

```python
outer = Outer()

a = outer.inner
b = outer.inner

assert a is b, f"a: {a}\nb: {b}"
```

```text
AssertionError: a: <builtins.Inner object at 0x00000238FFB9C7B0>
b: <builtins.Inner object at 0x00000238FFB9C830>
```

This can be especially confusing if the field is mutable, as getting the field and then mutating it won't persist - you'll just get a fresh clone of the original on the next access. Unfortunately Python and Rust don't agree about ownership - if PyO3 gave out references to (possibly) temporary Rust objects to Python code, Python code could then keep that reference alive indefinitely. Therefore returning Rust objects requires cloning.

If you don't want that cloning to happen, a workaround is to allocate the field on the Python heap and store a reference to that, by using [`Py<...>`]({{#PYO3_DOCS_URL}}/pyo3/struct.Py.html):
```rust
# use pyo3::prelude::*;
#[pyclass]
#[derive(Clone)]
struct Inner {/* fields omitted */}

#[pyclass]
struct Outer {
    #[pyo3(get)]
    inner: Py<Inner>,
}

#[pymethods]
impl Outer {
    #[new]
    fn __new__(py: Python<'_>) -> PyResult<Self> {
        Ok(Self {
            inner: Py::new(py, Inner {})?,
        })
    }
}
```
This time `a` and `b` *are* the same object:
```python
outer = Outer()

a = outer.inner
b = outer.inner

assert a is b, f"a: {a}\nb: {b}"
print(f"a: {a}\nb: {b}")
```

```text
a: <builtins.Inner object at 0x0000020044FCC670>
b: <builtins.Inner object at 0x0000020044FCC670>
```
The downside to this approach is that any Rust code working on the `Outer` struct now has to acquire the GIL to do anything with its field.

## I want to use the `pyo3` crate re-exported from from dependency but the proc-macros fail!

All PyO3 proc-macros (`#[pyclass]`, `#[pyfunction]`, `#[derive(FromPyObject)]`
and so on) expect the `pyo3` crate to be available under that name in your crate
root, which is the normal situation when `pyo3` is a direct dependency of your
crate.

However, when the dependency is renamed, or your crate only indirectly depends
on `pyo3`, you need to let the macro code know where to find the crate.  This is
done with the `crate` attribute:

```rust
# use pyo3::prelude::*;
# pub extern crate pyo3;
# mod reexported { pub use ::pyo3; }
#[pyclass]
#[pyo3(crate = "reexported::pyo3")]
struct MyClass;
```
