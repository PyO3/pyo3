# GIL lifetimes, mutability and Python object types

On first glance, PyO3 provides a huge number of different types that can be used
to wrap or refer to Python objects.  This page delves into the details and gives
an overview of their intended meaning, with examples when each type is best
used.


## Mutability and Rust types

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
  holding the GIL, such as [`&'py PyObject`][PyObject].

The latter two points are the reason why some APIs in PyO3 require the `py:
Python` argument, while others don't.

The PyO3 API for Python objects is written such that instead of requiring a
mutable Rust reference for mutating operations such as
[`PyList::append`][PyList_append], a shared reference (which, in turn, can only
be created through `Python<'_>` with a GIL lifetime) is sufficient.

However, Rust structs wrapped as Python objects (called `pyclass` types) usually
*do* need `&mut` access.  Due to the GIL, PyO3 *can* guarantee thread-safe acces
to them, but it cannot statically guarantee uniqueness of `&mut` references once
an object's ownership has been passed to the Python interpreter, ensuring
references is done at runtime using `PyCell`, a scheme very similar to
`std::cell::RefCell`.


## Object types

### [`PyObject`]

**Represents:** a Python object of unspecified type, restricted to a GIL
lifetime.  Currently, `PyObject` can only ever occur as a reference, `&PyObject`.

**Used:** Whenever you want to refer to some Python object and will have the
GIL for the whole duration you need to access that object. For example,
intermediate values and arguments to `pyfunction`s or `pymethod`s implemented
in Rust where any type is allowed.

Many general methods for interacting with Python objects are on the `PyObject` struct,
such as `getattr`, `setattr`, and `.call`.

**Conversions:**

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# let gil = Python::acquire_gil();
# let py = gil.python();
let obj: &PyObject = PyList::empty(py);

// Convert to &ConcreteType using PyObject::downcast
let _: &PyList = obj.downcast().unwrap();

// Convert to Py<PyObject> using .into() or Py::from
let _: Py<PyObject> = obj.into();

// Convert to Py<ConcreteType> using PyObject::extract
let _: Py<PyList> = obj.extract().unwrap();
```


### `PyTuple`, `PyDict`, and many more

**Represents:** a native Python object of known type, restricted to a GIL
lifetime just like `PyObject`.

**Used:** Whenever you want to operate with native Python types while holding
the GIL.  Like `PyObject`, this is the most convenient form to use for function
arguments and intermediate values.

These types all implement `Deref<Target = PyObject>`, so they all expose the same
methods which can be found on `PyObject`.

**Conversions:**

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# let gil = Python::acquire_gil();
# let py = gil.python();
let list = PyList::empty(py);

// Can use methods from PyObject on all Python types due to Deref implementation
let _ = list.repr();

// Rust will convert &PyList etc. to &PyObject automatically due to Deref implementation
let _: &PyObject = list;

// For more explicit &PyObject conversion, use .as_ref()
let _: &PyObject = list.as_ref();

// To convert to Py<T> use .into() or Py::from()
let _: Py<PyList> = list.into();
```

### `Py<SomeType>`

**Represents:** a GIL independent reference to a Python object of known type.
This can be a Python native type (like `PyTuple`), or a `pyclass` type
implemented in Rust.

**Used:** Whenever you want to carry around references to "some" Python object,
without caring about a GIL lifetime.  For example, storing Python object
references in a Rust struct that outlives the Python-Rust FFI boundary,
or returning objects from functions implemented in Rust back to Python.

**Conversions:**

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# let gil = Python::acquire_gil();
# let py = gil.python();
let list: Py<PyList> = PyList::empty(py).into();

// Access the native type using AsPyRef::as_ref(py)
// (For #[pyclass] types, as_ref() will return &PyCell<T>)
let _: &PyList = list.as_ref(py);
```

**Note:** `PyObject` is semantically equivalent to `Py<PyObject>` and might be
merged with it in the future.


### `PyCell<SomeType>`

**Represents:** a reference to a Rust object (instance of `PyClass`) which is
wrapped in a Python object.  The cell part is an analog to stdlib's
[`RefCell`][RefCell] to allow access to `&mut` references.

**Used:** for accessing pure-Rust API of the instance (members and functions
taking `&SomeType` or `&mut SomeType`) while maintaining the aliasing rules of
Rust references.

Like pyo3's Python native types, `PyCell<T>` implements `Deref<Target = PyObject>`,
so it also exposes all of the methods on `PyObject`.

**Conversions:**

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# #[pyclass] struct MyClass { }
# let gil = Python::acquire_gil();
# let py = gil.python();
let cell: &PyCell<MyClass> = PyCell::new(py, MyClass { }).unwrap();

// Obtain PyRef<T> with .try_borrow()
let pr: PyRef<MyClass> = cell.try_borrow().unwrap();
# drop(pr);

// Obtain PyRefMut<T> with .try_borrow_mut()
let prm: PyRefMut<MyClass> = cell.try_borrow_mut().unwrap();
# drop(prm);

// Can use methods from PyObject on PyCell<T> due to Deref implementation
let _ = cell.repr();

// Rust will convert &PyCell<T> to &PyObject automatically due to Deref implementation
let _: &PyObject = cell;

// For more explicit &PyObject conversion, use .as_ref()
let any: &PyObject = cell.as_ref();

// To obtain a PyCell<T> from PyObject, use PyObject::downcast
let _: &PyCell<MyClass> = any.downcast().unwrap();
```

### `PyRef<SomeType>` and `PyRefMut<SomeType>`

**Represents:** reference wrapper types employed by `PyCell` to keep track of
borrows, analog to `Ref` and `RefMut` used by `RefCell`.

**Used:** while borrowing a `PyCell`.  They can also be used with `.extract()`
on types like `Py<T>` and `PyObject` to get a reference quickly.



## Related traits and types

### `PyClass`

This trait marks structs defined in Rust that are also usable as Python classes,
usually defined using the `#[pyclass]` macro.

### `PyNativeType`

This trait marks structs that mirror native Python types, such as `PyList`.



[eval]: https://docs.rs/pyo3/latest/pyo3/struct.Python.html#method.eval
[clone_ref]: https://docs.rs/pyo3/latest/pyo3/struct.Py.html#method.clone_ref
[PyObject]: https://docs.rs/pyo3/latest/pyo3/types/struct.PyObject.html
[PyList_append]: https://docs.rs/pyo3/latest/pyo3/types/struct.PyList.html#method.append
[RefCell]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
