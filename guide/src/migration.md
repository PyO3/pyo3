# Migrating from older PyO3 versions

This guide can help you upgrade code through breaking changes from one PyO3 version to the next.
For a detailed list of all changes, see the [CHANGELOG](changelog.md).

## from 0.15.* to 0.16

### Drop support for older technologies

PyO3 0.16 has increased minimum Rust version to 1.48 and minimum Python version to 3.7. This enables use of newer language features (enabling some of the other additions in 0.16) and simplifies maintenance of the project.

### `#[pyproto]` has been deprecated

In PyO3 0.15, the `#[pymethods]` attribute macro gained support for implementing "magic methods" such as `__str__` (aka "dunder" methods). This implementation was not quite finalized at the time, with a few edge cases to be decided upon. The existing `#[pyproto]` attribute macro was left untouched, because it covered these edge cases.

In PyO3 0.16, the `#[pymethods]` implementation has been completed and is now the preferred way to implement magic methods. To allow the PyO3 project to move forward, `#[pyproto]` has been deprecated (with expected removal in PyO3 0.18).

Migration from `#[pyproto]` to `#[pymethods]` is straightforward; copying the existing methods directly from the `#[pyproto]` trait implementation is all that is needed in most cases.

Before:

```rust,ignore
use pyo3::prelude::*;
use pyo3::class::{PyBasicProtocol, PyIterProtocol};
use pyo3::types::PyString;

#[pyclass]
struct MyClass { }

#[pyproto]
impl PyBasicProtocol for MyClass {
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

```rust,ignore
use pyo3::prelude::*;
use pyo3::types::PyString;

#[pyclass]
struct MyClass { }

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

The `__len__` and `__getitem__` methods are also used to implement a Python [mapping](https://docs.python.org/3/glossary.html#term-mapping). In the Python C-API, these methods are not shared: the sequence `__len__` and `__getitem__` are defined by the `sq_len` and `sq_item` slots, and the mapping equivalents are `mp_len` and `mp_subscript`. There are similar distinctions for `__setitem__` and `__delitem__`.

Because there is no such distinction from Python, implementing these methods will fill the mapping and sequence slots simultaneously. A Python class with `__len__` implemented, for example, will have both the `sq_len` and `mp_len` slots filled.

The PyO3 behavior in 0.16 has been changed to be closer to this Python behavior by default.

### `wrap_pymodule!` now respects privacy correctly

Prior to PyO3 0.16 the `wrap_pymodule!` macro could use modules declared in Rust modules which were not reachable.

For example, the following code was legal before 0.16, but in 0.16 is rejected because the `wrap_pymodule!` macro cannot access the `private_submodule` function:

```rust,compile_fail
mod foo {
    use pyo3::prelude::*;

    #[pymodule]
    fn private_submodule(_py: Python, m: &PyModule) -> PyResult<()> {
        Ok(())
    }
}

use pyo3::prelude::*;
use foo::*;

#[pymodule]
fn my_module(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pymodule!(private_submodule))?;
    Ok(())
}
```

To fix it, make the private submodule visible, e.g. with `pub` or `pub(crate)`.

```rust
mod foo {
    use pyo3::prelude::*;

    #[pymodule]
    pub(crate) fn private_submodule(_py: Python, m: &PyModule) -> PyResult<()> {
        Ok(())
    }
}

use pyo3::prelude::*;
use pyo3::wrap_pymodule;
use foo::*;

#[pymodule]
fn my_module(_py: Python, m: &PyModule) -> PyResult<()> {
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

Some protocol (aka `__dunder__`) methods such as `__bytes__` and `__format__` have been possible to implement two ways in PyO3 for some time: via a `#[pyproto]` (e.g. `PyBasicProtocol` for the methods listed here), or by writing them directly in `#[pymethods]`. This is only true for a handful of the `#[pyproto]` methods (for technical reasons to do with the way PyO3 currently interacts with the Python C-API).

In the interest of having onle one way to do things, the `#[pyproto]` forms of these methods have been deprecated.

To migrate just move the affected methods from a `#[pyproto]` to a `#[pymethods]` block.

Before:

```rust,ignore
use pyo3::prelude::*;
use pyo3::class::basic::PyBasicProtocol;

#[pyclass]
struct MyClass { }

#[pyproto]
impl PyBasicProtocol for MyClass {
    fn __bytes__(&self) -> &'static [u8] {
        b"hello, world"
    }
}
```

After:

```rust
use pyo3::prelude::*;

#[pyclass]
struct MyClass { }

#[pymethods]
impl MyClass {
    fn __bytes__(&self) -> &'static [u8] {
        b"hello, world"
    }
}
```

## from 0.12.* to 0.13

### Minimum Rust version increased to Rust 1.45

PyO3 `0.13` makes use of new Rust language features stabilised between Rust 1.40 and Rust 1.45. If you are using a Rust compiler older than Rust 1.45, you will need to update your toolchain to be able to continue using PyO3.

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
```rust,ignore
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
assert_eq!(instance.getattr("__class__")?, PyTypeError::type_object(py).as_ref());
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
```rust,ignore
# use pyo3::prelude::*;
struct MyPyObjectWrapper(PyObject);

impl FromPy<MyPyObjectWrapper> for PyObject {
    fn from_py(other: MyPyObjectWrapper, _py: Python) -> Self {
        other.0
    }
}
```

After
```rust
# use pyo3::prelude::*;
struct MyPyObjectWrapper(PyObject);

impl IntoPy<PyObject> for MyPyObjectWrapper {
    fn into_py(self, _py: Python) -> PyObject {
        self.0
    }
}
```

Similarly, code which was using the `FromPy` trait can be trivially rewritten to use `IntoPy`.

Before:
```rust,ignore
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
```rust
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
       closure: Box<dyn Fn()>
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
       closure: Box<dyn Fn() + Send>
   }
   ```

   In situations where you cannot change your `#[pyclass]` to automatically implement `Send`
   (e.g., when it contains a raw pointer), you can use `unsafe impl Send`.
   In such cases, care should be taken to ensure the struct is actually thread safe.
   See [the Rustnomicon](https://doc.rust-lang.org/nomicon/send-and-sync.html) for more.

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
```rust,compile_fail
use pyo3::ObjectProtocol;

# pyo3::Python::with_gil(|py| {
let obj = py.eval("lambda: 'Hi :)'", None, None).unwrap();
let hi: &pyo3::types::PyString = obj.call0().unwrap().downcast().unwrap();
assert_eq!(hi.len().unwrap(), 5);
# })
```

After:
```rust
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
       obj.init(MyClass { })
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
    names: Vec<String>
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
#     let names = PyCell::new(py, Names::new()).unwrap();
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
```rust
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
```ignore
let obj: &PyAny = create_obj();
let obj_ref: &MyClass = obj.extract().unwrap();
let obj_ref_mut: &mut MyClass = obj.extract().unwrap();
```

After:
```rust
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
    let obj_ref: PyRef<MyClass> = obj.extract().unwrap();
    // we need to drop obj_ref before we can extract a PyRefMut due to Rust's rules of references
}
let obj_ref_mut: PyRefMut<MyClass> = obj.extract().unwrap();
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
```rust,ignore
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
