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
  holding the GIL, such as [`&'py PyAny`][PyAny].

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

### [`PyAny`]

**Represents:** a Python object of unspecified type, restricted to a GIL
lifetime.  Currently, `PyAny` can only ever occur as a reference, `&PyAny`.

**Used:** Whenever you want to refer to some Python object and will have the
GIL for the whole duration you need to access that object. For example,
intermediate values and arguments to `pyfunction`s or `pymethod`s implemented
in Rust where any type is allowed.

Many general methods for interacting with Python objects are on the `PyAny` struct,
such as `getattr`, `setattr`, and `.call`.

**Conversions:**

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# let gil = Python::acquire_gil();
# let py = gil.python();
let obj: &PyAny = PyList::empty(py);

// Convert to &ConcreteType using PyAny::downcast
let _: &PyList = obj.downcast().unwrap();

// Convert to Py<PyAny> using .into() or Py::from
let _: Py<PyAny> = obj.into();

// Convert to Py<ConcreteType> using PyAny::extract
let _: Py<PyList> = obj.extract().unwrap();
```


### `PyTuple`, `PyDict`, and many more

**Represents:** a native Python object of known type, restricted to a GIL
lifetime just like `PyAny`.

**Used:** Whenever you want to operate with native Python types while holding
the GIL.  Like `PyAny`, this is the most convenient form to use for function
arguments and intermediate values.

These types all implement `Deref<Target = PyAny>`, so they all expose the same
methods which can be found on `PyAny`.

**Conversions:**

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# let gil = Python::acquire_gil();
# let py = gil.python();
let list = PyList::empty(py);

// Can use methods from PyAny on all Python types due to Deref implementation
let _ = list.repr();

// Rust will convert &PyList etc. to &PyAny automatically due to Deref implementation
let _: &PyAny = list;

// For more explicit &PyAny conversion, use .as_ref()
let _: &PyAny = list.as_ref();

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

**Note:** `PyObject` is semantically equivalent to `Py<PyAny>` and might be
merged with it in the future.


### `PyCell<SomeType>`

**Represents:** a reference to a Rust object (instance of `PyClass`) which is
wrapped in a Python object.  The cell part is an analog to stdlib's
[`RefCell`][RefCell] to allow access to `&mut` references.

**Used:** for accessing pure-Rust API of the instance (members and functions
taking `&SomeType` or `&mut SomeType`) while maintaining the aliasing rules of
Rust references.

Like pyo3's Python native types, `PyCell<T>` implements `Deref<Target = PyAny>`,
so it also exposes all of the methods on `PyAny`.

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

// Can use methods from PyAny on PyCell<T> due to Deref implementation
let _ = cell.repr();

// Rust will convert &PyCell<T> to &PyAny automatically due to Deref implementation
let _: &PyAny = cell;

// For more explicit &PyAny conversion, use .as_ref()
let any: &PyAny = cell.as_ref();

// To obtain a PyCell<T> from PyAny, use PyAny::downcast
let _: &PyCell<MyClass> = any.downcast().unwrap();
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



[eval]: https://docs.rs/pyo3/latest/pyo3/struct.Python.html#method.eval
[clone_ref]: https://docs.rs/pyo3/latest/pyo3/struct.Py.html#method.clone_ref
[PyAny]: https://docs.rs/pyo3/latest/pyo3/types/struct.PyAny.html
[PyList_append]: https://docs.rs/pyo3/latest/pyo3/types/struct.PyList.html#method.append
[RefCell]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
