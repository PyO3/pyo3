# GIL lifetimes, mutability and Python object types

On first glance, PyO3 provides a huge number of different types that can be used
to wrap or refer to Python objects.  This page delves into the details and gives
an overview of their intended meaning, with examples when each type is best
used.


## The Python GIL, mutability, and Rust types

Since Python has no concept of ownership, and works solely with boxed objects,
any Python object can be referenced any number of times, and mutation is allowed
from any reference.

The situation is helped a little by the Global Interpreter Lock (GIL), which
ensures that only one thread can use the Python interpreter and its API at the
same time, while non-Python operations (system calls and extension code) can
unlock the GIL.  (See [the section on parallelism](parallelism.md) for how to do
that in PyO3.)

In PyO3, holding the GIL is modeled by acquiring a token of the type
`Python<'py>`, which serves three purposes:

* It provides some global API for the Python interpreter, such as
  [`eval`][eval].
* It can be passed to functions that require a proof of holding the GIL,
  such as [`Py::clone_ref`][clone_ref].
* Its lifetime can be used to create Rust references that implicitly guarantee
  holding the GIL, such as [`&'py PyAny`][PyAny].

The latter two points are the reason why some APIs in PyO3 require the `py:
Python` argument, while others don't.

The PyO3 API for Python objects is written such that instead of requiring a
mutable Rust reference for mutating operations such as
[`PyList::append`][PyList_append], a shared reference (which, in turn, can only
be created through `Python<'_>` with a GIL lifetime) is sufficient.

However, Rust structs wrapped as Python objects (called `pyclass` types) usually
*do* need `&mut` access.  Due to the GIL, PyO3 *can* guarantee thread-safe access
to them, but it cannot statically guarantee uniqueness of `&mut` references once
an object's ownership has been passed to the Python interpreter, ensuring
references is done at runtime using `PyCell`, a scheme very similar to
`std::cell::RefCell`.

### Accessing the Python GIL

To get hold of a `Python<'py>` token to prove the GIL is held, consult [PyO3's documentation][obtaining-py].

## Object types

### [`PyAny`][PyAny]

**Represents:** a Python object of unspecified type, restricted to a GIL
lifetime.  Currently, `PyAny` can only ever occur as a reference, `&PyAny`.

**Used:** Whenever you want to refer to some Python object and will have the
GIL for the whole duration you need to access that object. For example,
intermediate values and arguments to `pyfunction`s or `pymethod`s implemented
in Rust where any type is allowed.

Many general methods for interacting with Python objects are on the `PyAny` struct,
such as `getattr`, `setattr`, and `.call`.

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
# #[allow(deprecated)]
let obj: &PyAny = Py::new(py, MyClass {})?.into_ref(py);

// To &PyCell<MyClass> with PyAny::downcast
# #[allow(deprecated)]
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


## Related traits and types

### `PyClass`

This trait marks structs defined in Rust that are also usable as Python classes,
usually defined using the `#[pyclass]` macro.

### `PyNativeType`

This trait marks structs that mirror native Python types, such as `PyList`.


[eval]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.eval
[clone_ref]: {{#PYO3_DOCS_URL}}/pyo3/struct.Py.html#method.clone_ref
[pyo3::types]: {{#PYO3_DOCS_URL}}/pyo3/types/index.html
[PyAny]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyAny.html
[PyList_append]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyList.html#method.append
[RefCell]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
[obtaining-py]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#obtaining-a-python-token
