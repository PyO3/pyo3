# Python object types

PyO3 offers two main sets of types to interact with Python objects. This section of the guide expands into detail about these types and how to choose which to use.

The first set of types is are the "smart pointers" which all Python objects are wrapped in. These are `Py<T>`, `Bound<'py, T>`, and `Borrowed<'a, 'py, T>`. The [first section below](#pyo3s-smart-pointers) expands on each of these in detail and why there are three of them.

The second set of types are types which fill in the generic parameter `T` of the smart pointers. The most common is `PyAny`, which represents any Python object (similar to Python's `typing.Any`). There are also concrete types for many Python built-in types, such as `PyList`, `PyDict`, and `PyTuple`. User defined `#[pyclass]` types also fit this category. The [second section below](#concrete-python-types) expands on how to use these types.

Before PyO3 0.21, PyO3's main API to interact with Python objects was a deprecated API known as the "GIL Refs" API, containing reference types such as `&PyAny`, `&PyList`, and `&PyCell<T>` for user-defined `#[pyclass]` types. The [third section below](#the-gil-refs-api) details this deprecated API.

## PyO3's smart pointers

PyO3's API offers three generic smart pointers: `Py<T>`, `Bound<'py, T>` and `Borrowed<'a, 'py, T>`. For each of these the type parameter `T` will be filled by a [concrete Python type](#concrete-python-types). For example, a Python list object can be represented by `Py<PyList>`, `Bound<'py, PyList>`, and `Borrowed<'a, 'py, PyList>`.

These smart pointers have different numbers of lifetime parameters, which defines how they behave. `Py<T>` has no lifetime parameters, `Bound<'py, T>` has a lifetime parameter `'py`, and `Borrowed<'a, 'py, T>` has two lifetime parameters `'a` and `'py`.

Python objects are reference counted, like [`std::sync::Arc`](https://doc.rust-lang.org/stable/std/sync/struct.Arc.html). A major reason for these smart pointers is to bring Python's reference counting to a Rust API.

The recommendation of when to use each of these smart pointers is as follows:

- Use `Bound<'py, T>` for as much as possible, as it offers the most efficient and complete API.
- Use `Py<T>` mostly just for storage inside Rust `struct`s which do not want to add a lifetime parameter for `Bound<'py, T>`.
- `Borrowed<'a, 'py, T>` is almost never used. It is occasionally present at the boundary between Rust and the Python interpreter, for example when borrowing data from Python tuples (which is safe because they are immutable).

The sections below also explain these smart pointers in a little more detail.

### `Py<T>` (and `PyObject`)

[`Py<T>`][Py] is the foundational smart pointer in PyO3's API. The type parameter `T` denotes the type of the Python object. Very frequently this is `PyAny`, meaning any Python object. This is so common that `Py<PyAny>` has a type alias `PyObject`.

Because `Py<T>` is not bound to [the `'py` lifetime](./python-from-rust.md#the-py-lifetime), it is the type to use when storing a Python object inside a Rust `struct` or `enum` which do not want to have a lifetime parameter. In particular, [`#[pyclass]`][pyclass] types are not permitted to have a lifetime, so `Py<T>` is the correct type to store Python objects inside them.

The lack of binding to the `'py` lifetime also carries drawbacks:
 - Almost all methods on `Py<T>` require a `Python<'py>` token as the first argument
 - Other functionality, such as [`Drop`][Drop], needs to check at runtime for attachment to the Python GIL, at a small performance cost

Because of the drawbacks `Bound<'py, T>` is preferred for many of PyO3's APIs.

To convert a `Py<T>` into a `Bound<'py, T>`, the `Py::bind` and `Py::into_bound` methods are available. `Bound<'py, T>` can be converted back into `Py<T>` using `Bound::unbind`.

### `Bound<'py, T>`

[`Bound<'py, T>`][Bound] is the counterpart to `Py<T>` which is also bound to the `'py` lifetime. It can be thought of as equivalent to the Rust tuple `(Python<'py>, Py<T>)`.

By having the binding to the `'py` lifetime, `Bound<'py, T>` can offer the complete PyO3 API at maximum efficiency. This means that in almost all cases where `Py<T>` is not necessary for lifetime reasons, `Bound<'py, T>` should be used.

`Bound<'py, T>` engages in Python reference counting. This means that `Bound<'py, T>` owns a Python object. Rust code which just wants to borrow a Python object should use a shared reference `&Bound<'py, T>`. Just like `std::sync::Arg`, using `.clone()` and `drop()` will cheaply implement and decrement the reference count of the object (just in this case, the reference counting is implemented by the Python interpreter itself).

To give an example of how `Bound<'py, T>` is PyO3's primary API type, consider the following Python code:

```python
def example():
    x = list()   # create a Python list
    x.append(1)  # append the integer 1 to it
    y = x        # create a second reference to the list
    del x        # delete the original reference
```

Using PyO3's API, and in particular `Bound<'py, PyList>`, this code translates into the following Rust code:

```rust
use pyo3::prelude::*;
use pyo3::types::PyList;

# fn main() -> PyResult<()> {
// the 'py lifetime doesn't need to be named explicitly in this code,
// so '_ is used.
Python::with_gil(|py: Python<'_>| -> PyResult<()> {
    let x: Bound<'_, PyList> = PyList::empty_bound(py);
    x.append(1)?;
    let y: Bound<'_, PyList> = x.clone();  // y is a new reference to the same list
    drop(x);                               // release the original reference x
    Ok(())
})?;  // y dropped implicitly here at the end of the closure
# Ok(())
# }
```

Or, without the type annotations:

```rust
use pyo3::prelude::*;
use pyo3::types::PyList;

# fn main() -> PyResult<()> {
Python::with_gil(|py| {
    let x = PyList::empty_bound(py);
    x.append(1)?;
    let y = x.clone();
    drop(x);
    Ok(())
})?;
# Ok(())
# }
```

### `Borrowed<'a, 'py, T>`

[`Borrowed<'a, 'py, T>`][Borrowed] is an advanced type used just occasionally at the edge of interaction with the Python interpreter. It can be thought of as analogous to the shared reference `&'a Bound<'py, T>`. The difference is that `Borrowed<'a, 'py, T>` is just a smart pointer rather than a reference-to-a-smart-pointer, which is a helpful reduction in indirection in specific interactions with the Python interpreter.

`Borrowed<'a, 'py, T>` dereferences to `Bound<'py, T>`, so all methods on `Bound<'py, T>` are available on `Borrowed<'a, 'py, T>`.

An example where `Borrowed<'a, 'py, T>` is used is in [`PyTupleMethods::get_borrowed_item`](#{{PYO3_DOCS_URL}}/pyo3/types/trait.PyTupleMethods.html#tymethod.get_item):

```rust
use pyo3::prelude::*;
use pyo3::types::PyTuple;

# fn main() -> PyResult<()> {
Python::with_gil(|py| {
    // Create a new tuple with the elements (0, 1, 2)
    let t = PyTuple::new_bound(py, 0..=2);
    for i in 0..=2 {
        let entry: Borrowed<'_, '_, PyAny> = t.get_borrowed_item(i)?;
        // `PyAnyMethods::extract` is available on `Borrowed`
        // via the dereference to `Bound`
        let value: usize = entry.extract()?;
        assert_eq!(i, value);
    }
})?;
# Ok(())
# }
```

## Concrete Python types

In all of `Py<T>`, `Bound<'py, T>`, and `Borrowed<'a, 'py, T>`, the type parameter `T` denotes the type of the Python object referred to by the smart pointer.

This parameter `T` can be filled by:
 - [`PyAny`][PyAny], which represents any Python object,
 - Native Python types such as `PyList`, `PyTuple`, and `PyDict`, and
 - [`#[pyclass]`][pyclass] types defined from Rust

The following subsections elaborate on working with these types in a little more detail.

### [`PyAny`][PyAny]

A Python object of unspecified type.

**Uses:** Whenever you want to refer to some Python object without knowing its type.

Many general methods for interacting with Python objects are on the `Bound<'py, PyAny>` smart pointer, such as `.getattr`, `.setattr`, and `.call`.

**Conversions:**

For a `Bound<'py, PyAny>` where the underlying object is a Python-native type such as a list:

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# fn example<'py>(py: Python<'py>) -> PyResult<()> {
let obj: Bound<'py, PyAny> = PyList::empty_bound(py).into_any();

// To &Bound<'py, PyList> with PyAnyMethods::downcast
let _: &Bound<'py, PyList> = obj.downcast()?;

# let obj2 = obj.clone();
// To Py<PyAny> (aka PyObject) with .unbind()
let _: Py<PyAny> = obj.unbind();
# let obj = obj2;

// To Bound<'py, PyList> with PyAnyMethods::downcast_into
let list: Bound<'py, PyList> = obj.downcast_into()?;
# Ok(())
# }
# Python::with_gil(example).unwrap()
```

For a `Bound<'_, PyAny>` where the underlying object is a `#[pyclass]`:

```rust
# use pyo3::prelude::*;
# #[pyclass] #[derive(Clone)] struct MyClass { }
# fn example<'py>(py: Python<'py>) -> PyResult<()> {
let obj: Bound<'_, PyAny> = Bound::new(py, MyClass {})?.into_any();

// To &Bound<'py, MyClass> with PyAnyMethods::downcast
let _: &Bound<'_, MyClass> = obj.downcast()?;

# let obj2 = obj.clone();
// To Py<PyAny> (aka PyObject) with .unbind()
let _: Py<PyAny> = obj.unbind();
# let obj = obj2;

// To Bound<'_, MyClass> with PyAnyMethods::downcast_into
let _: Bound<'_, MyClass> = obj.downcast_into()?;

// To MyClass with PyAny::extract, if MyClass: Clone
let _: MyClass = obj.extract()?;

// To PyRef<'_, MyClass> or PyRefMut<'_, MyClass> with PyAny::extract
let _: PyRef<'_, MyClass> = obj.extract()?;
let _: PyRefMut<'_, MyClass> = obj.extract()?;
# Ok(())
# }
# Python::with_gil(example).unwrap()
```

### `PyTuple`, `PyDict`, and many more

**Represents:** a native Python object of known type, restricted to a GIL
lifetime just like `PyAny`.

**Used:** Whenever you want to operate with native Python types while holding
the GIL.  Like `PyAny`, this is the most convenient form to use for function
arguments and intermediate values.

These types all implement `Deref<Target = PyAny>`, so they all expose the same
methods which can be found on `PyAny`.

To see all Python types exposed by `PyO3` you should consult the
[`pyo3::types`][pyo3::types] module.

**Conversions:**

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# Python::with_gil(|py| -> PyResult<()> {
#[allow(deprecated)]  // PyList::empty is part of the deprecated "GIL Refs" API.
let list = PyList::empty(py);

// Use methods from PyAny on all Python types with Deref implementation
let _ = list.repr()?;

// To &PyAny automatically with Deref implementation
let _: &PyAny = list;

// To &PyAny explicitly with .as_ref()
#[allow(deprecated)]  // as_ref is part of the deprecated "GIL Refs" API.
let _: &PyAny = list.as_ref();

// To Py<T> with .into() or Py::from()
let _: Py<PyList> = list.into();

// To PyObject with .into() or .to_object(py)
let _: PyObject = list.into();
# Ok(())
# }).unwrap();
```

### `Py<T>` and `PyObject`

**Represents:** a GIL-independent reference to a Python object. This can be a Python native type
(like `PyTuple`), or a `pyclass` type implemented in Rust. The most commonly-used variant,
`Py<PyAny>`, is also known as `PyObject`.

**Used:** Whenever you want to carry around references to a Python object without caring about a
GIL lifetime.  For example, storing Python object references in a Rust struct that outlives the
Python-Rust FFI boundary, or returning objects from functions implemented in Rust back to Python.

Can be cloned using Python reference counts with `.clone()`.

**Conversions:**

For a `Py<PyList>`, the conversions are as below:

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# Python::with_gil(|py| {
let list: Py<PyList> = PyList::empty_bound(py).unbind();

// To &PyList with Py::as_ref() (borrows from the Py)
#[allow(deprecated)]  // as_ref is part of the deprecated "GIL Refs" API.
let _: &PyList = list.as_ref(py);

# let list_clone = list.clone(); // Because `.into_ref()` will consume `list`.
// To &PyList with Py::into_ref() (moves the pointer into PyO3's object storage)
# #[allow(deprecated)]
let _: &PyList = list.into_ref(py);

# let list = list_clone;
// To Py<PyAny> (aka PyObject) with .into()
let _: Py<PyAny> = list.into();
# })
```

For a `#[pyclass] struct MyClass`, the conversions for `Py<MyClass>` are below:

```rust
# use pyo3::prelude::*;
# Python::with_gil(|py| {
# #[pyclass] struct MyClass { }
# Python::with_gil(|py| -> PyResult<()> {
let my_class: Py<MyClass> = Py::new(py, MyClass { })?;

// To &PyCell<MyClass> with Py::as_ref() (borrows from the Py)
#[allow(deprecated)]  // as_ref is part of the deprecated "GIL Refs" API.
let _: &PyCell<MyClass> = my_class.as_ref(py);

# let my_class_clone = my_class.clone(); // Because `.into_ref()` will consume `my_class`.
// To &PyCell<MyClass> with Py::into_ref() (moves the pointer into PyO3's object storage)
# #[allow(deprecated)]
let _: &PyCell<MyClass> = my_class.into_ref(py);

# let my_class = my_class_clone.clone();
// To Py<PyAny> (aka PyObject) with .into_py(py)
let _: Py<PyAny> = my_class.into_py(py);

# let my_class = my_class_clone;
// To PyRef<'_, MyClass> with Py::borrow or Py::try_borrow
let _: PyRef<'_, MyClass> = my_class.try_borrow(py)?;

// To PyRefMut<'_, MyClass> with Py::borrow_mut or Py::try_borrow_mut
let _: PyRefMut<'_, MyClass> = my_class.try_borrow_mut(py)?;
# Ok(())
# }).unwrap();
# });
```

### `PyCell<SomeType>`

**Represents:** a reference to a Rust object (instance of `PyClass`) which is
wrapped in a Python object.  The cell part is an analog to stdlib's
[`RefCell`][RefCell] to allow access to `&mut` references.

**Used:** for accessing pure-Rust API of the instance (members and functions
taking `&SomeType` or `&mut SomeType`) while maintaining the aliasing rules of
Rust references.

Like PyO3's Python native types, `PyCell<T>` implements `Deref<Target = PyAny>`,
so it also exposes all of the methods on `PyAny`.

**Conversions:**

`PyCell<T>` can be used to access `&T` and `&mut T` via `PyRef<T>` and `PyRefMut<T>` respectively.

```rust
# use pyo3::prelude::*;
# #[pyclass] struct MyClass { }
# Python::with_gil(|py| -> PyResult<()> {
# #[allow(deprecated)]
let cell: &PyCell<MyClass> = PyCell::new(py, MyClass {})?;

// To PyRef<T> with .borrow() or .try_borrow()
let py_ref: PyRef<'_, MyClass> = cell.try_borrow()?;
let _: &MyClass = &*py_ref;
# drop(py_ref);

// To PyRefMut<T> with .borrow_mut() or .try_borrow_mut()
let mut py_ref_mut: PyRefMut<'_, MyClass> = cell.try_borrow_mut()?;
let _: &mut MyClass = &mut *py_ref_mut;
# Ok(())
# }).unwrap();
```

`PyCell<T>` can also be accessed like a Python-native type.

```rust
# use pyo3::prelude::*;
# #[pyclass] struct MyClass { }
# Python::with_gil(|py| -> PyResult<()> {
# #[allow(deprecated)]
let cell: &PyCell<MyClass> = PyCell::new(py, MyClass {})?;

// Use methods from PyAny on PyCell<T> with Deref implementation
let _ = cell.repr()?;

// To &PyAny automatically with Deref implementation
let _: &PyAny = cell;

// To &PyAny explicitly with .as_ref()
#[allow(deprecated)]  // as_ref is part of the deprecated "GIL Refs" API.
let _: &PyAny = cell.as_ref();
# Ok(())
# }).unwrap();
```

### `PyRef<SomeType>` and `PyRefMut<SomeType>`

**Represents:** reference wrapper types employed by `PyCell` to keep track of
borrows, analog to `Ref` and `RefMut` used by `RefCell`.

**Used:** while borrowing a `PyCell`.  They can also be used with `.extract()`
on types like `Py<T>` and `PyAny` to get a reference quickly.

## The GIL Refs API

The GIL Refs API was PyO3's primary API prior to PyO3 0.21. The main difference was that instead of the `Bound<'py, PyAny>` smart pointer, the "GIL Reference" `&'py PyAny` was used. (This was similar for other Python types.)

As of PyO3 0.21, the GIL Refs API is deprecated. See the [migration guide](./migration.md#from-020-to-021) for details on how to upgrade.

The following sections note some historical detail about the GIL Refs API.

### [`PyAny`][PyAny]

**Represented:** a Python object of unspecified type. In the GIL Refs API, this was only accessed as the GIL Ref `&'py PyAny`.

**Used:** `&'py PyAny` was used to refer to some Python object when the GIL lifetime was available for the whole duration access was needed. For example, intermediate values and arguments to `pyfunction`s or `pymethod`s implemented in Rust where any type is allowed.

**Conversions:**

For a `&PyAny` object reference `any` where the underlying object is a Python-native type such as
a list:

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# Python::with_gil(|py| -> PyResult<()> {
#[allow(deprecated)]  // PyList::empty is part of the deprecated "GIL Refs" API.
let obj: &PyAny = PyList::empty(py);

// To &PyList with PyAny::downcast
let _: &PyList = obj.downcast()?;

// To Py<PyAny> (aka PyObject) with .into()
let _: Py<PyAny> = obj.into();

// To Py<PyList> with PyAny::extract
let _: Py<PyList> = obj.extract()?;
# Ok(())
# }).unwrap();
```

For a `&PyAny` object reference `any` where the underlying object is a `#[pyclass]`:

```rust
# use pyo3::prelude::*;
# #[pyclass] #[derive(Clone)] struct MyClass { }
# Python::with_gil(|py| -> PyResult<()> {
#[allow(deprecated)]  // into_ref is part of the deprecated GIL Refs API
let obj: &PyAny = Py::new(py, MyClass {})?.into_ref(py);

// To &PyCell<MyClass> with PyAny::downcast
#[allow(deprecated)]  // &PyCell is part of the deprecated GIL Refs API
let _: &PyCell<MyClass> = obj.downcast()?;

// To Py<PyAny> (aka PyObject) with .into()
let _: Py<PyAny> = obj.into();

// To Py<MyClass> with PyAny::extract
let _: Py<MyClass> = obj.extract()?;

// To MyClass with PyAny::extract, if MyClass: Clone
let _: MyClass = obj.extract()?;

// To PyRef<'_, MyClass> or PyRefMut<'_, MyClass> with PyAny::extract
let _: PyRef<'_, MyClass> = obj.extract()?;
let _: PyRefMut<'_, MyClass> = obj.extract()?;
# Ok(())
# }).unwrap();
```

### `PyTuple`, `PyDict`, and many more

**Represented:** a native Python object of known type. In the GIL Refs API, they were only accessed as the GIL Refs `&'py PyTuple`, `&'py PyDict`.

**Used:** `&'py PyTuple` and similar were used to operate with native Python types while holding the GIL. Like `PyAny`, this is the most convenient form to use for function arguments and intermediate values.

These GIL Refs implement `Deref<Target = PyAny>`, so they all expose the same methods which can be found on `PyAny`.

To see all Python types exposed by `PyO3` consult the [`pyo3::types`][pyo3::types] module.

**Conversions:**

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# Python::with_gil(|py| -> PyResult<()> {
#[allow(deprecated)]  // PyList::empty is part of the deprecated "GIL Refs" API.
let list = PyList::empty(py);

// Use methods from PyAny on all Python types with Deref implementation
let _ = list.repr()?;

// To &PyAny automatically with Deref implementation
let _: &PyAny = list;

// To &PyAny explicitly with .as_ref()
#[allow(deprecated)]  // as_ref is part of the deprecated "GIL Refs" API.
let _: &PyAny = list.as_ref();

// To Py<T> with .into() or Py::from()
let _: Py<PyList> = list.into();

// To PyObject with .into() or .to_object(py)
let _: PyObject = list.into();
# Ok(())
# }).unwrap();
```

### `Py<T>` and `PyObject`

**Represented:** a GIL-independent reference to a Python object. This can be a Python native type
(like `PyTuple`), or a `pyclass` type implemented in Rust. The most commonly-used variant,
`Py<PyAny>`, is also known as `PyObject`.

**Used:** Whenever you want to carry around references to a Python object without caring about a
GIL lifetime.  For example, storing Python object references in a Rust struct that outlives the
Python-Rust FFI boundary, or returning objects from functions implemented in Rust back to Python.

Can be cloned using Python reference counts with `.clone()`.

### `PyCell<SomeType>`

**Represented:** a reference to a Rust object (instance of `PyClass`) wrapped in a Python object.  The cell part is an analog to stdlib's [`RefCell`][RefCell] to allow access to `&mut` references.

**Used:** for accessing pure-Rust API of the instance (members and functions taking `&SomeType` or `&mut SomeType`) while maintaining the aliasing rules of Rust references.

Like PyO3's Python native types, the GIL Ref `&PyCell<T>` implements `Deref<Target = PyAny>`, so it also exposed all of the methods on `PyAny`.

**Conversions:**

`PyCell<T>` was used to access `&T` and `&mut T` via `PyRef<T>` and `PyRefMut<T>` respectively.

```rust
# use pyo3::prelude::*;
# #[pyclass] struct MyClass { }
# Python::with_gil(|py| -> PyResult<()> {
#[allow(deprecated)]  // &PyCell is part of the deprecated GIL Refs API
let cell: &PyCell<MyClass> = PyCell::new(py, MyClass {})?;

// To PyRef<T> with .borrow() or .try_borrow()
let py_ref: PyRef<'_, MyClass> = cell.try_borrow()?;
let _: &MyClass = &*py_ref;
# drop(py_ref);

// To PyRefMut<T> with .borrow_mut() or .try_borrow_mut()
let mut py_ref_mut: PyRefMut<'_, MyClass> = cell.try_borrow_mut()?;
let _: &mut MyClass = &mut *py_ref_mut;
# Ok(())
# }).unwrap();
```

`PyCell<T>` was also accessed like a Python-native type.

```rust
# use pyo3::prelude::*;
# #[pyclass] struct MyClass { }
# Python::with_gil(|py| -> PyResult<()> {
#[allow(deprecated)]  // &PyCell is part of the deprecate GIL Refs API
let cell: &PyCell<MyClass> = PyCell::new(py, MyClass {})?;

// Use methods from PyAny on PyCell<T> with Deref implementation
let _ = cell.repr()?;

// To &PyAny automatically with Deref implementation
let _: &PyAny = cell;

// To &PyAny explicitly with .as_ref()
#[allow(deprecated)]  // as_ref is part of the deprecated "GIL Refs" API.
let _: &PyAny = cell.as_ref();
# Ok(())
# }).unwrap();
```

[pyclass]: class.md
[Py]: {{#PYO3_DOCS_URL}}/pyo3/struct.Py.html
[Bound]: {{#PYO3_DOCS_URL}}/pyo3/struct.Bound.html
[Borrowed]: {{#PYO3_DOCS_URL}}/pyo3/struct.Borrowed.html
[Drop]: https://doc.rust-lang.org/std/drop/trait.Drop.html
[eval]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.eval
[clone_ref]: {{#PYO3_DOCS_URL}}/pyo3/struct.Py.html#method.clone_ref
[pyo3::types]: {{#PYO3_DOCS_URL}}/pyo3/types/index.html
[PyAny]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyAny.html
[PyList_append]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyList.html#method.append
[RefCell]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
