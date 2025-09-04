# Python object types

PyO3 offers two main sets of types to interact with Python objects. This section of the guide expands into detail about these types and how to choose which to use.

The first set of types are the [smart pointers][smart-pointers] which all Python objects are wrapped in. These are `Py<T>`, `Bound<'py, T>`, and `Borrowed<'a, 'py, T>`. The [first section below](#pyo3s-smart-pointers) expands on each of these in detail and why there are three of them.

The second set of types are types which fill in the generic parameter `T` of the smart pointers. The most common is `PyAny`, which represents any Python object (similar to Python's `typing.Any`). There are also concrete types for many Python built-in types, such as `PyList`, `PyDict`, and `PyTuple`. User defined `#[pyclass]` types also fit this category. The [second section below](#concrete-python-types) expands on how to use these types.

## PyO3's smart pointers

PyO3's API offers three generic smart pointers: `Py<T>`, `Bound<'py, T>` and `Borrowed<'a, 'py, T>`. For each of these the type parameter `T` will be filled by a [concrete Python type](#concrete-python-types). For example, a Python list object can be represented by `Py<PyList>`, `Bound<'py, PyList>`, and `Borrowed<'a, 'py, PyList>`.

These smart pointers behave differently due to their lifetime parameters. `Py<T>` has no lifetime parameters, `Bound<'py, T>` has [the `'py` lifetime](./python-from-rust.md#the-py-lifetime) as a parameter, and `Borrowed<'a, 'py, T>` has the `'py` lifetime plus an additional lifetime `'a` to denote the lifetime it is borrowing data for. (You can read more about these lifetimes in the subsections below).

Python objects are reference counted, like [`std::sync::Arc`](https://doc.rust-lang.org/stable/std/sync/struct.Arc.html). A major reason for these smart pointers is to bring Python's reference counting to a Rust API.

The recommendation of when to use each of these smart pointers is as follows:

- Use `Bound<'py, T>` for as much as possible, as it offers the most efficient and complete API.
- Use `Py<T>` mostly just for storage inside Rust `struct`s which do not want to or can't add a lifetime parameter for `Bound<'py, T>`.
- `Borrowed<'a, 'py, T>` is almost never used. It is occasionally present at the boundary between Rust and the Python interpreter, for example when borrowing data from Python tuples (which is safe because they are immutable).

The sections below also explain these smart pointers in a little more detail.

### `Py<T>`

[`Py<T>`][Py] is the foundational smart pointer in PyO3's API. The type parameter `T` denotes the type of the Python object. Very frequently this is `PyAny`, meaning any Python object.

Because `Py<T>` is not bound to [the `'py` lifetime](./python-from-rust.md#the-py-lifetime), it is the type to use when storing a Python object inside a Rust `struct` or `enum` which do not want to have a lifetime parameter. In particular, [`#[pyclass]`][pyclass] types are not permitted to have a lifetime, so `Py<T>` is the correct type to store Python objects inside them.

The lack of binding to the `'py` lifetime also carries drawbacks:
 - Almost all methods on `Py<T>` require a `Python<'py>` token as the first argument
 - Other functionality, such as [`Drop`][Drop], needs to check at runtime for attachment to the Python interpreter, at a small performance cost

Because of the drawbacks `Bound<'py, T>` is preferred for many of PyO3's APIs. In particular, `Bound<'py, T>` is better for function arguments.

To convert a `Py<T>` into a `Bound<'py, T>`, the `Py::bind` and `Py::into_bound` methods are available. `Bound<'py, T>` can be converted back into `Py<T>` using [`Bound::unbind`].

### `Bound<'py, T>`

[`Bound<'py, T>`][Bound] is the counterpart to `Py<T>` which is also bound to the `'py` lifetime. It can be thought of as equivalent to the Rust tuple `(Python<'py>, Py<T>)`.

By having the binding to the `'py` lifetime, `Bound<'py, T>` can offer the complete PyO3 API at maximum efficiency. This means that `Bound<'py, T>` should usually be used whenever carrying this lifetime is acceptable, and `Py<T>` otherwise.

`Bound<'py, T>` engages in Python reference counting. This means that `Bound<'py, T>` owns a Python object. Rust code which just wants to borrow a Python object should use a shared reference `&Bound<'py, T>`. Just like `std::sync::Arc`, using `.clone()` and `drop()` will cheaply increment and decrement the reference count of the object (just in this case, the reference counting is implemented by the Python interpreter itself).

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
    let x: Bound<'py, PyList> = PyList::empty(py);
    x.append(1)?;
    let y: Bound<'py, PyList> = x.clone(); // y is a new reference to the same list
    drop(x); // release the original reference x
    Ok(())
}
# Python::attach(example).unwrap();
```

Or, without the type annotations:

```rust
use pyo3::prelude::*;
use pyo3::types::PyList;

fn example(py: Python<'_>) -> PyResult<()> {
    let x = PyList::empty(py);
    x.append(1)?;
    let y = x.clone();
    drop(x);
    Ok(())
}
# Python::attach(example).unwrap();
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
fn add<'py>(
    left: &Bound<'py, PyAny>,
    right: &Bound<'py, PyAny>,
) -> PyResult<Bound<'py, PyAny>> {
    left.add(right)
}
# Python::attach(|py| {
#     let s = pyo3::types::PyString::new(py, "s");
#     assert!(add(&s, &s).unwrap().eq("ss").unwrap());
# })
```

If naming the `'py` lifetime adds unwanted complexity to the function signature, it is also acceptable to return `Py<PyAny>`, which has no lifetime. The cost is instead paid by a slight increase in implementation complexity, as seen by the introduction of a call to [`Bound::unbind`]:

```rust
# use pyo3::prelude::*;
fn add(left: &Bound<'_, PyAny>, right: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let output: Bound<'_, PyAny> = left.add(right)?;
    Ok(output.unbind())
}
# Python::attach(|py| {
#     let s = pyo3::types::PyString::new(py, "s");
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
let t = PyTuple::new(py, [0, 1, 2])?;
for i in 0..=2 {
    let entry: Borrowed<'_, 'py, PyAny> = t.get_borrowed_item(i)?;
    // `PyAnyMethods::extract` is available on `Borrowed`
    // via the dereference to `Bound`
    let value: usize = entry.extract()?;
    assert_eq!(i, value);
}
# Ok(())
# }
# Python::attach(example).unwrap();
```

### Casting between smart pointer types

To convert between `Py<T>` and `Bound<'py, T>` use the `bind()` / `into_bound()` methods. Use the `as_unbound()` / `unbind()` methods to go back from `Bound<'py, T>` to `Py<T>`.

```rust,ignore
let obj: Py<PyAny> = ...;
let bound: &Bound<'py, PyAny> = obj.bind(py);
let bound: Bound<'py, PyAny> = obj.into_bound(py);

let obj: &Py<PyAny> = bound.as_unbound();
let obj: Py<PyAny> = bound.unbind();
```

To convert between `Bound<'py, T>` and `Borrowed<'a, 'py, T>` use the `as_borrowed()` method. `Borrowed<'a, 'py, T>` has a deref coercion to `Bound<'py, T>`. Use the `to_owned()` method to increment the Python reference count and to create a new `Bound<'py, T>` from the `Borrowed<'a, 'py, T>`.

```rust,ignore
let bound: Bound<'py, PyAny> = ...;
let borrowed: Borrowed<'_, 'py, PyAny> = bound.as_borrowed();

// deref coercion
let bound: &Bound<'py, PyAny> = &borrowed;

// create a new Bound by increase the Python reference count
let bound: Bound<'py, PyAny> = borrowed.to_owned();
```

To convert between `Py<T>` and `Borrowed<'a, 'py, T>` use the `bind_borrowed()` method. Use either `as_unbound()` or `.to_owned().unbind()` to go back to `Py<T>` from `Borrowed<'a, 'py, T>`, via `Bound<'py, T>`.

```rust,ignore
let obj: Py<PyAny> = ...;
let borrowed: Borrowed<'_, 'py, PyAny> = bound.as_borrowed();

// via deref coercion to Bound and then using Bound::as_unbound
let obj: &Py<PyAny> = borrowed.as_unbound();

// via a new Bound by increasing the Python reference count, and unbind it
let obj: Py<PyAny> = borrowed.to_owned().unbind().
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
- Consistency: downstream code implementing Rust APIs for existing Python types can also follow this pattern of using a trait. Downstream code would not be allowed to add new associated methods directly on the `Bound` type.
- Future design: it is hoped that a future Rust with [arbitrary self types](https://github.com/rust-lang/rust/issues/44874) will remove the need for these traits in favour of placing the methods directly on `PyAny`, `PyTuple`, `PyDict`, and so on.

These traits are all included in the `pyo3::prelude` module, so with the glob import `use pyo3::prelude::*` the full PyO3 API is made available to downstream code.

The following function accesses the first item in the input Python list, using the `.get_item()` method from the `PyListMethods` trait:

```rust
use pyo3::prelude::*;
use pyo3::types::PyList;

fn get_first_item<'py>(list: &Bound<'py, PyList>) -> PyResult<Bound<'py, PyAny>> {
    list.get_item(0)
}
# Python::attach(|py| {
#     let l = PyList::new(py, ["hello world"]).unwrap();
#     assert!(get_first_item(&l).unwrap().eq("hello world").unwrap());
# })
```

### Casting between Python object types

To cast `Bound<'py, T>` smart pointers to some other type, use the [`.cast()`][Bound::cast] family of functions. This converts `&Bound<'py, T>` to a different `&Bound<'py, U>`, without transferring ownership. There is also [`.cast_into()`][Bound::cast_into] to convert `Bound<'py, T>` to `Bound<'py, U>` with transfer of ownership. These methods are available for all types `T` which implement the [`PyTypeCheck`] trait.

Casting to `Bound<'py, PyAny>` can be done with `.as_any()` or `.into_any()`.

For example, the following snippet shows how to cast `Bound<'py, PyAny>` to `Bound<'py, PyTuple>`:

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyTuple;
# fn example<'py>(py: Python<'py>) -> PyResult<()> {
// create a new Python `tuple`, and use `.into_any()` to erase the type
let obj: Bound<'py, PyAny> = PyTuple::empty(py).into_any();

// use `.cast()` to cast to `PyTuple` without transferring ownership
let _: &Bound<'py, PyTuple> = obj.cast()?;

// use `.cast_into()` to cast to `PyTuple` with transfer of ownership
let _: Bound<'py, PyTuple> = obj.cast_into()?;
# Ok(())
# }
# Python::attach(example).unwrap()
```

Custom [`#[pyclass]`][pyclass] types implement [`PyTypeCheck`], so `.cast()` also works for these types. The snippet below is the same as the snippet above casting instead to a custom type `MyClass`:

```rust
use pyo3::prelude::*;

#[pyclass]
struct MyClass {}

# fn example<'py>(py: Python<'py>) -> PyResult<()> {
// create a new Python `tuple`, and use `.into_any()` to erase the type
let obj: Bound<'py, PyAny> = Bound::new(py, MyClass {})?.into_any();

// use `.cast()` to cast to `MyClass` without transferring ownership
let _: &Bound<'py, MyClass> = obj.cast()?;

// use `.cast_into()` to cast to `MyClass` with transfer of ownership
let _: Bound<'py, MyClass> = obj.cast_into()?;
# Ok(())
# }
# Python::attach(example).unwrap()
```

### Extracting Rust data from Python objects

To extract Rust data from Python objects, use [`.extract()`][PyAnyMethods::extract] instead of `.cast()`. This method is available for all types which implement the [`FromPyObject`] trait.

For example, the following snippet extracts a Rust tuple of integers from a Python tuple:

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyTuple;
# fn example<'py>(py: Python<'py>) -> PyResult<()> {
// create a new Python `tuple`, and use `.into_any()` to erase the type
let obj: Bound<'py, PyAny> = PyTuple::new(py, [1, 2, 3])?.into_any();

// extracting the Python `tuple` to a rust `(i32, i32, i32)` tuple
let (x, y, z) = obj.extract::<(i32, i32, i32)>()?;
assert_eq!((x, y, z), (1, 2, 3));
# Ok(())
# }
# Python::attach(example).unwrap()
```

To avoid copying data, [`#[pyclass]`][pyclass] types can directly reference Rust data stored within the Python objects without needing to `.extract()`. See the [corresponding documentation in the class section of the guide](./class.md#bound-and-interior-mutability)
for more detail.

[Bound]: {{#PYO3_DOCS_URL}}/pyo3/struct.Bound.html
[`Bound::unbind`]: {{#PYO3_DOCS_URL}}/pyo3/struct.Bound.html#method.unbind
[Py]: {{#PYO3_DOCS_URL}}/pyo3/struct.Py.html
[PyAnyMethods::add]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.add
[PyAnyMethods::extract]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.extract
[Bound::cast]: {{#PYO3_DOCS_URL}}/pyo3/struct.Bound.html#method.cast
[Bound::cast_into]: {{#PYO3_DOCS_URL}}/pyo3/struct.Bound.html#method.cast_into
[`PyTypeCheck`]: {{#PYO3_DOCS_URL}}/pyo3/type_object/trait.PyTypeCheck.html
[`PyAnyMethods`]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html
[`PyDictMethods`]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyDictMethods.html
[`PyTupleMethods`]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyTupleMethods.html
[pyclass]: class.md
[Borrowed]: {{#PYO3_DOCS_URL}}/pyo3/struct.Borrowed.html
[Drop]: https://doc.rust-lang.org/std/ops/trait.Drop.html
[eval]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.eval
[clone_ref]: {{#PYO3_DOCS_URL}}/pyo3/struct.Py.html#method.clone_ref
[pyo3::types]: {{#PYO3_DOCS_URL}}/pyo3/types/index.html
[PyAny]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyAny.html
[PyList_append]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyList.html#method.append
[RefCell]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
[smart-pointers]: https://doc.rust-lang.org/book/ch15-00-smart-pointers.html
