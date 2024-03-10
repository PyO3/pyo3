# Python object types

PyO3 offers two main sets of types to interact with Python objects. This section of the guide expands into detail about these types and how to choose which to use.

The first set of types is are the "smart pointers" which all Python objects are wrapped in. These are `Py<T>`, `Bound<'py, T>`, and `Borrowed<'a, 'py, T>`. The [first section below](#pyo3s-smart-pointers) expands on each of these in detail and why there are three of them.

The second set of types are types which fill in the generic parameter `T` of the smart pointers. The most common is `PyAny`, which represents any Python object (similar to Python's `typing.Any`). There are also concrete types for many Python built-in types, such as `PyList`, `PyDict`, and `PyTuple`. User defined `#[pyclass]` types also fit this category. The [second section below](#concrete-python-types) expands on how to use these types.

Before PyO3 0.21, PyO3's main API to interact with Python objects was a deprecated API known as the "GIL Refs" API, containing reference types such as `&PyAny`, `&PyList`, and `&PyCell<T>` for user-defined `#[pyclass]` types. The [third section below](#the-gil-refs-api) details this deprecated API.

## PyO3's smart pointers

PyO3's API offers three generic smart pointers: `Py<T>`, `Bound<'py, T>` and `Borrowed<'a, 'py, T>`. For each of these the type parameter `T` will be filled by a [concrete Python type](#concrete-python-types). For example, a Python list object can be represented by `Py<PyList>`, `Bound<'py, PyList>`, and `Borrowed<'a, 'py, PyList>`.

These smart pointers behave differently due to their lifetime parameters. `Py<T>` has no lifetime parameters, `Bound<'py, T>` has [the `'py` lifetime](./python-from-rust.md#the-py-lifetime) as a parameter, and `Borrowed<'a, 'py, T>` has the `'py` lifetime plus an additional lifetime `'a` to denote the lifetime it is borrowing data for. (You can read more about these lifetimes in the subsections below).

Python objects are reference counted, like [`std::sync::Arc`](https://doc.rust-lang.org/stable/std/sync/struct.Arc.html). A major reason for these smart pointers is to bring Python's reference counting to a Rust API.

The recommendation of when to use each of these smart pointers is as follows:

- Use `Bound<'py, T>` for as much as possible, as it offers the most efficient and complete API.
- Use `Py<T>` mostly just for storage inside Rust `struct`s which do not want to or can't add a lifetime parameter for `Bound<'py, T>`.
- `Borrowed<'a, 'py, T>` is almost never used. It is occasionally present at the boundary between Rust and the Python interpreter, for example when borrowing data from Python tuples (which is safe because they are immutable).

The sections below also explain these smart pointers in a little more detail.

### `Py<T>` (and `PyObject`)

[`Py<T>`][Py] is the foundational smart pointer in PyO3's API. The type parameter `T` denotes the type of the Python object. Very frequently this is `PyAny`, meaning any Python object. This is so common that `Py<PyAny>` has a type alias `PyObject`.

Because `Py<T>` is not bound to [the `'py` lifetime](./python-from-rust.md#the-py-lifetime), it is the type to use when storing a Python object inside a Rust `struct` or `enum` which do not want to have a lifetime parameter. In particular, [`#[pyclass]`][pyclass] types are not permitted to have a lifetime, so `Py<T>` is the correct type to store Python objects inside them.

The lack of binding to the `'py` lifetime also carries drawbacks:
 - Almost all methods on `Py<T>` require a `Python<'py>` token as the first argument
 - Other functionality, such as [`Drop`][Drop], needs to check at runtime for attachment to the Python GIL, at a small performance cost

Because of the drawbacks `Bound<'py, T>` is preferred for many of PyO3's APIs. In particular, `Bound<'py, T>` is the better for function arguments.

To convert a `Py<T>` into a `Bound<'py, T>`, the `Py::bind` and `Py::into_bound` methods are available. `Bound<'py, T>` can be converted back into `Py<T>` using [`Bound::unbind`].

### `Bound<'py, T>`

[`Bound<'py, T>`][Bound] is the counterpart to `Py<T>` which is also bound to the `'py` lifetime. It can be thought of as equivalent to the Rust tuple `(Python<'py>, Py<T>)`.

By having the binding to the `'py` lifetime, `Bound<'py, T>` can offer the complete PyO3 API at maximum efficiency. This means that in almost all cases where `Py<T>` is not necessary for lifetime reasons, `Bound<'py, T>` should be used.

`Bound<'py, T>` engages in Python reference counting. This means that `Bound<'py, T>` owns a Python object. Rust code which just wants to borrow a Python object should use a shared reference `&Bound<'py, T>`. Just like `std::sync::Arc`, using `.clone()` and `drop()` will cheaply implement and decrement the reference count of the object (just in this case, the reference counting is implemented by the Python interpreter itself).

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

fn example<'py>(py: Python<'py>) -> PyResult<()> {
    let x: Bound<'py, PyList> = PyList::empty_bound(py);
    x.append(1)?;
    let y: Bound<'py, PyList> = x.clone();  // y is a new reference to the same list
    drop(x);                               // release the original reference x
    Ok(())
}
# Python::with_gil(example).unwrap();
```

Or, without the type annotations:

```rust
use pyo3::prelude::*;
use pyo3::types::PyList;

# fn example(py: Python<'_>) -> PyResult<()> {
    let x = PyList::empty_bound(py);
    x.append(1)?;
    let y = x.clone();
    drop(x);
    Ok(())
}
# Python::with_gil(example).unwrap();
```

#### Function argument lifetimes

Because the `'py` lifetime often appears in many function arguments as part of the `Bound<'py, T>` smart pointer, the Rust compiler will often require annotations of input and output lifetimes. This occurs when the function output has at least one lifetime, and there is more than one lifetime present on the inputs.

To demonstrate, consider this function which takes accepts Python objects and applies the [Python `+` operation][PyAnyMethods::add] to them:

```rust,compile_fail
# use pyo3::prelude::*;
fn add(left: &'_ Bound<'_, PyAny>, right: &'_ Bound<'_, PyAny>) -> PyResult<Bound<'_, PyAny>> {
    left.add(right)
}
```

Because the Python `+` operation might raise an exception, this function returns `PyResult<Bound<'_, PyAny>>`. It doesn't need ownership of the inputs, so it takes `&Bound<'_, PyAny>` shared references. To demonstrate the point, all lifetimes have used the wildcard `'_` to allow the Rust compiler to attempt to infer them. Because there are four input lifetimes (two lifetimes of the shared references, and two `'py` lifetimes unnamed inside the `Bound<'_, PyAny>` pointers), the compiler cannot reason about which must be connected to the output.

The correct way to solve this is to add the `'py` lifetime as a parameter for the function, and name all the `'py` lifetimes inside the `Bound<'py, PyAny>` smart pointers. For the shared references, it's also fine to reduce `&'_` to just `&`. The working end result is below:

```rust
# use pyo3::prelude::*;
fn add<'py>(left: &Bound<'py, PyAny>, right: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    left.add(right)
}
# Python::with_gil(|py| {
#     let s = pyo3::types::PyString::new_bound(py, "s");
#     assert!(add(&s, &s).unwrap().eq("ss").unwrap());
# })
```

If naming the `'py` lifetime adds unwanted complexity to the function signature, it is also acceptable to return `PyObject` (aka `Py<PyAny>`), which has no lifetime. The cost is instead paid by a slight increase in implementation complexity, as seen by the introduction of a call to [`Bound::unbind`]:

```rust
# use pyo3::prelude::*;
fn add(left: &Bound<'_, PyAny>, right: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let output: Bound<'_, PyAny> = left.add(right)?;
    Ok(output.unbind())
}
# Python::with_gil(|py| {
#     let s = pyo3::types::PyString::new_bound(py, "s");
#     assert!(add(&s, &s).unwrap().bind(py).eq("ss").unwrap());
# })
```

### `Borrowed<'a, 'py, T>`

[`Borrowed<'a, 'py, T>`][Borrowed] is an advanced type used just occasionally at the edge of interaction with the Python interpreter. It can be thought of as analogous to the shared reference `&'a Bound<'py, T>`. The difference is that `Borrowed<'a, 'py, T>` is just a smart pointer rather than a reference-to-a-smart-pointer, which is a helpful reduction in indirection in specific interactions with the Python interpreter.

`Borrowed<'a, 'py, T>` dereferences to `Bound<'py, T>`, so all methods on `Bound<'py, T>` are available on `Borrowed<'a, 'py, T>`.

An example where `Borrowed<'a, 'py, T>` is used is in [`PyTupleMethods::get_borrowed_item`]({{#PYO3_DOCS_URL}}/pyo3/types/trait.PyTupleMethods.html#tymethod.get_item):

```rust
use pyo3::prelude::*;
use pyo3::types::PyTuple;

# fn example<'py>(py: Python<'py>) -> PyResult<()> {
// Create a new tuple with the elements (0, 1, 2)
let t = PyTuple::new_bound(py, [0, 1, 2]);
for i in 0..=2 {
    let entry: Borrowed<'_, 'py, PyAny> = t.get_borrowed_item(i)?;
    // `PyAnyMethods::extract` is available on `Borrowed`
    // via the dereference to `Bound`
    let value: usize = entry.extract()?;
    assert_eq!(i, value);
}
# Ok(())
# }
# Python::with_gil(example).unwrap();
```

## Concrete Python types

In all of `Py<T>`, `Bound<'py, T>`, and `Borrowed<'a, 'py, T>`, the type parameter `T` denotes the type of the Python object referred to by the smart pointer.

This parameter `T` can be filled by:
 - [`PyAny`][PyAny], which represents any Python object,
 - Native Python types such as `PyList`, `PyTuple`, and `PyDict`, and
 - [`#[pyclass]`][pyclass] types defined from Rust

The following subsections covers some further detail about how to work with these types:
- the APIs that are available for these concrete types,
- how to cast `Bound<'py, T>` to a specific concrete type, and
- how to get Rust data out of a `Bound<'py, T>`.

### Using APIs for concrete Python types

Each concrete Python type such as `PyAny`, `PyTuple` and `PyDict` exposes its API on the corresponding bound smart pointer `Bound<'py, PyAny>`, `Bound<'py, PyTuple>` and `Bound<'py, PyDict>`.

Each type's API is exposed as a trait: [`PyAnyMethods`], [`PyTupleMethods`], [`PyDictMethods`], and so on for all concrete types. Using traits rather than associated methods on the `Bound` smart pointer is done for a couple of reasons:
- Clarity of documentation: each trait gets its own documentation page in the PyO3 API docs. If all methods were on the `Bound` smart pointer directly, the vast majority of PyO3's API would be on a single, extremely long, documentation page.
- Consistency: downstream code implementing [Rust APIs for existing Python types](#creating-a-rust-api-for-an-existing-python-type) can also follow this pattern of using a trait. Downstream code would not be allowed to add new associated methods directly on the `Bound` type.
- Future design: it is hoped that a future Rust with [arbitrary self types](TODO) will remove the need for these traits in favour of placing the methods directly on `PyAny`, `PyTuple`, `PyDict`, and so on.

These traits are all included in the `pyo3::prelude` module, so with the glob import `use pyo3::prelude::*` the full PyO3 API is made available to downstream code.

The following function accesses the first item in the input Python list, using the `.get_item()` method from the `PyListMethods` trait:

```rust
use pyo3::prelude::*;
use pyo3::types::PyList;

fn get_first_item<'py>(list: &Bound<'py, PyList>) -> PyResult<Bound<'py, PyAny>> {
    list.get_item(0)
}
# Python::with_gil(|py| {
#     let l = PyList::new_bound(py, ["hello world"]);
#     assert!(get_first_item(&l).unwrap().eq("hello world").unwrap());
# })
```

### Casting between Python object types

To cast `Bound<'py, T>` smart pointers to some other type, use the [`.downcast()`][PyAnyMethods::downcast] family of functions. This converts `&Bound<'py, T>` to a different `&Bound<'py, U>`, without transferring ownership. There is also [`.downcast_into()`][PyAnyMethods::downcast_into] to convert `Bound<'py, T>` to `Bound<'py, U>` with transfer of ownership. These methods are available for all types `T` which implement the [`PyTypeCheck`] trait.

Casting to `Bound<'py, PyAny>` can be done with `.as_any()` or `.into_any()`.

For example, the following snippet shows how to cast `Bound<'py, PyAny>` to `Bound<'py, PyTuple>`:

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyTuple;
# fn example<'py>(py: Python<'py>) -> PyResult<()> {
// create a new Python `tuple`, and use `.into_any()` to erase the type
let obj: Bound<'py, PyAny> = PyTuple::empty_bound(py).into_any();

// use `.downcast()` to cast to `PyTuple` without transferring ownership
let _: &Bound<'py, PyTuple> = obj.downcast()?;

// use `.downcast_into()` to cast to `PyTuple` with transfer of ownership
let _: Bound<'py, PyTuple> = obj.downcast_into()?;
# Ok(())
# }
# Python::with_gil(example).unwrap()
```

Custom [`#[pyclass]`][pyclass] types implement [`PyTypeCheck`], so `.downcast()` also works for these types. The snippet below is the same as the snippet above casting instead to a custom type `MyClass`:

```rust
use pyo3::prelude::*;

#[pyclass]
struct MyClass { }

# fn example<'py>(py: Python<'py>) -> PyResult<()> {
// create a new Python `tuple`, and use `.into_any()` to erase the type
let obj: Bound<'py, PyAny> = Bound::new(py, MyClass { })?.into_any();

// use `.downcast()` to cast to `MyClass` without transferring ownership
let _: &Bound<'py, MyClass> = obj.downcast()?;

// use `.downcast_into()` to cast to `MyClass` with transfer of ownership
let _: Bound<'py, MyClass> = obj.downcast_into()?;
# Ok(())
# }
# Python::with_gil(example).unwrap()
```

### Extracting Rust data from Python objects

To extract Rust data from Python objects, use [`.extract()`][PyAnyMethods::extract] instead of `.downcast()`. This method is available for all types which implement the [`FromPyObject`] trait.

For example, the following snippet extracts a Rust tuple of integers from a Python tuple:

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyTuple;
# fn example<'py>(py: Python<'py>) -> PyResult<()> {
// create a new Python `tuple`, and use `.into_any()` to erase the type
let obj: Bound<'py, PyAny> = PyTuple::new_bound(py, [1, 2, 3]).into_any();

// extracting the Python `tuple` to a rust `(i32, i32, i32)` tuple
let (x, y, z) = obj.extract::<(i32, i32, i32)>()?;
assert_eq!((x, y, z), (1, 2, 3));
# Ok(())
# }
# Python::with_gil(example).unwrap()
```

To avoid copying data, [`#[pyclass]`][pyclass] types can directly reference Rust data stored within the Python objects without needing to `.extract()`. See the [corresponding documentation in the class section of the guide](./class.
md#bound-and-interior-mutability) for more detail.

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

[Bound]: {{#PYO3_DOCS_URL}}/pyo3/struct.Bound.html
[`Bound::unbind`]: {{#PYO3_DOCS_URL}}/pyo3/struct.Bound.html#method.unbind
[Py]: {{#PYO3_DOCS_URL}}/pyo3/struct.Py.html
[PyAnyMethods::add]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.add
[PyAnyMethods::extract]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.extract
[PyAnyMethods::downcast]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.downcast
[PyAnyMethods::downcast_into]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.downcast_into
[`PyTypeCheck`]: {{#PYO3_DOCS_URL}}/pyo3/type_object/trait.PyTypeCheck.html
[`PyAnyMethods`]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html
[`PyDictMethods`]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyDictMethods.html
[`PyTupleMethods`]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyTupleMethods.html
[pyclass]: class.md
[Borrowed]: {{#PYO3_DOCS_URL}}/pyo3/struct.Borrowed.html
[Drop]: https://doc.rust-lang.org/std/drop/trait.Drop.html
[eval]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.eval
[clone_ref]: {{#PYO3_DOCS_URL}}/pyo3/struct.Py.html#method.clone_ref
[pyo3::types]: {{#PYO3_DOCS_URL}}/pyo3/types/index.html
[PyAny]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyAny.html
[PyList_append]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyList.html#method.append
[RefCell]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
