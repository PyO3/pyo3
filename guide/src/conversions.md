# Type Conversions

In this portion of the guide we'll talk about the mapping of Python types to Rust types offered by PyO3, as well as the traits available to perform conversions between them.

## Mapping of Rust types to Python types

When writing functions callable from Python (such as a `#[pyfunction]` or in a `#[pymethods]` block), the trait `FromPyObject` is required for function arguments, and `IntoPy<PyObject>` is required for function return values.

Consult the tables in the following section to find the Rust types provided by PyO3 which implement these traits.

### Argument Types

When accepting a function argument, it is possible to either use Rust library types or PyO3's Python-native types. (See the next section for discussion on when to use each.)

The table below contains the Python type and the corresponding function argument types that will accept them:

| Python        | Rust                            | Rust (Python-native) |
| ------------- |:-------------------------------:|:--------------------:|
| `object`      | -                               | `&PyAny`             |
| `str`         | `String`, `Cow<str>`, `&str`    | `&PyUnicode`         |
| `bytes`       | `Vec<u8>`, `&[u8]`              | `&PyBytes`           |
| `bool`        | `bool`                          | `&PyBool`            |
| `int`         | Any integer type (`i32`, `u32`, `usize`, etc) | `&PyLong` |
| `float`       | `f32`, `f64`                    | `&PyFloat`           |
| `complex`     | `num_complex::Complex`[^1]      | `&PyComplex`         |
| `list[T]`     | `Vec<T>`                        | `&PyList`            |
| `dict[K, V]`  | `HashMap<K, V>`, `BTreeMap<K, V>` | `&PyDict`          |
| `tuple[T, U]` | `(T, U)`, `Vec<T>`              | `&PyTuple`           |
| `set[T]`      | `HashSet<T>`, `BTreeSet<T>`     | `&PySet`             |
| `frozenset[T]` | `HashSet<T>`, `BTreeSet<T>`    | `&PyFrozenSet`       |
| `bytearray`   | `Vec<u8>`                       | `&PyByteArray`       |
| `slice`       | -                               | `&PySlice`           |
| `type`        | -                               | `&PyType`            |
| `module`      | -                               | `&PyModule`          |
| `datetime.datetime` | -                         | `&PyDateTime`        |
| `datetime.date` | -                             | `&PyDate`            |
| `datetime.time` | -                             | `&PyTime`            |
| `datetime.tzinfo` | -                           | `&PyTzInfo`          |
| `datetime.timedelta` | -                        | `&PyDelta`           |
| `typing.Optional[T]` | `Option<T>`              | -                    |
| `typing.Sequence[T]` | `Vec<T>`                 | `&PySequence`        |
| `typing.Iterator[Any]` | -                      | `&PyIterator`        |

There are also a few special types related to the GIL and Rust-defined `#[pyclass]`es which may come in useful:

| What          | Description |
| ------------- | ------------------------------- |
| `Python`      | A GIL token, used to pass to PyO3 constructors to prove ownership of the GIL |
| `PyObject`    | A Python object isolated from the GIL lifetime. This can be sent to other threads. To call Python APIs using this object, it must be used with `AsPyRef::as_ref` to get a `&PyAny` reference. |
| `Py<T>`       | Same as above, for a specific Python type or `#[pyclass]` T. |
| `&PyCell<T>`  | A `#[pyclass]` value owned by Python. |
| `PyRef<T>`    | A `#[pyclass]` borrowed immutably. |
| `PyRefMut<T>` | A `#[pyclass]` borrowed mutably. |

For more detail on accepting `#[pyclass]` values as function arguments, see [the section of this guide on Python Classes](class.md).

#### Using Rust library types vs Python-native types

Using Rust library types as function arguments will incur a conversion cost compared to using the Python-native types. Using the Python-native types is almost zero-cost (they just require a type check similar to the Python builtin function `isinstance()`).

However, once that conversion cost has been paid, the Rust standard library types offer a number of benefits:
- You can write functionality in native-speed Rust code (free of Python's runtime costs).
- You get better interoperability with the rest of the Rust ecosystem.
- You can use `Python::allow_threads` to release the Python GIL and let other Python threads make progress while your Rust code is executing.
- You also benefit from stricter type checking. For example you can specify `Vec<i32>`, which will only accept a Python `list` containing integers. The Python-native equivalent, `&PyList`, would accept a Python `list` containing Python objects of any type.

For most PyO3 usage the conversion cost is worth paying to get these benefits. As always, if you're not sure it's worth it in your case, benchmark it!

### Returning Rust values to Python

When returning values from functions callable from Python, Python-native types (`&PyAny`, `&PyDict` etc.) can be used with zero cost.

Because these types are references, in some situations the Rust compiler may ask for lifetime annotations. If this is the case, you should use `Py<PyAny>`, `Py<PyDict>` etc. instead - which are also zero-cost and can be created from the native types with an `.into()` conversion.

If your function is fallible, it should return `PyResult<T>`, which will raise a `Python` exception if the `Err` variant is returned.

Finally, the following Rust types are also able to convert to Python as return values:

| Rust type     | Resulting Python Type           |
| ------------- |:-------------------------------:|
| `String`      | `str`                           |
| `&str`        | `str`                           |
| `bool`        | `bool`                          |
| Any integer type (`i32`, `u32`, `usize`, etc) | `int` |
| `f32`, `f64`  | `float`                         |
| `Option<T>`   | `Optional[T]`                   |
| `(T, U)`      | `Tuple[T, U]`                   |
| `Vec<T>`      | `List[T]`                       |
| `HashMap<K, V>` | `Dict[K, V]`                  |
| `BTreeMap<K, V>` | `Dict[K, V]`                 |
| `HashSet<T>`  | `Set[T]`                        |
| `BTreeSet<T>` | `Set[T]`                        |
| `&PyCell<T: PyClass>` | `T`                     |
| `PyRef<T: PyClass>` | `T`                       |
| `PyRefMut<T: PyClass>` | `T`                    |

## Traits

PyO3 provides some handy traits to convert between Python types and Rust types.

### `.extract()` and the `FromPyObject` trait

The easiest way to convert a Python object to a Rust value is using
`.extract()`.  It returns a `PyResult` with a type error if the conversion
fails, so usually you will use something like

```ignore
let v: Vec<i32> = obj.extract()?;
```

This method is available for many Python object types, and can produce a wide
variety of Rust types, which you can check out in the implementor list of
[`FromPyObject`].

[`FromPyObject`] is also implemented for your own Rust types wrapped as Python
objects (see [the chapter about classes](class.md)).  There, in order to both be
able to operate on mutable references *and* satisfy Rust's rules of non-aliasing
mutable references, you have to extract the PyO3 reference wrappers [`PyRef`]
and [`PyRefMut`].  They work like the reference wrappers of
`std::cell::RefCell` and ensure (at runtime) that Rust borrows are allowed.


### The `ToPyObject` trait

[`ToPyObject`] is a conversion trait that allows various objects to be
converted into [`PyObject`]. `IntoPy<PyObject>` serves the
same purpose, except that it consumes `self`.


### `*args` and `**kwargs` for Python object calls

There are several ways how to pass positional and keyword arguments to a Python object call.
[`PyAny`] provides two methods:

* `call` - call any callable Python object.
* `call_method` - call a specific method on the object, shorthand for `get_attr` then `call`.

Both methods need `args` and `kwargs` arguments, but there are variants for less
complex calls, such as `call1` for only `args` and `call0` for no arguments at all.

```rust
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

struct SomeObject;
impl SomeObject {
    fn new(py: Python) -> PyObject {
        PyDict::new(py).to_object(py)
    }
}

fn main() {
    let arg1 = "arg1";
    let arg2 = "arg2";
    let arg3 = "arg3";

    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = SomeObject::new(py);

    // call object without empty arguments
    obj.call0(py);

    // call object with PyTuple
    let args = PyTuple::new(py, &[arg1, arg2, arg3]);
    obj.call1(py, args);

    // pass arguments as rust tuple
    let args = (arg1, arg2, arg3);
    obj.call1(py, args);
}
```

`kwargs` can be `None` or `Some(&PyDict)`. You can use the
[`IntoPyDict`] trait to convert other dict-like containers,
e.g. `HashMap` or `BTreeMap`, as well as tuples with up to 10 elements and
`Vec`s where each element is a two-element tuple.

```rust
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict};
use std::collections::HashMap;

struct SomeObject;

impl SomeObject {
    fn new(py: Python) -> PyObject {
        PyDict::new(py).to_object(py)
    }
}

fn main() {
    let key1 = "key1";
    let val1 = 1;
    let key2 = "key2";
    let val2 = 2;

    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = SomeObject::new(py);

    // call object with PyDict
    let kwargs = [(key1, val1)].into_py_dict(py);
    obj.call(py, (), Some(kwargs));

    // pass arguments as Vec
    let kwargs = vec![(key1, val1), (key2, val2)];
    obj.call(py, (), Some(kwargs.into_py_dict(py)));

    // pass arguments as HashMap
    let mut kwargs = HashMap::<&str, i32>::new();
    kwargs.insert(key1, 1);
    obj.call(py, (), Some(kwargs.into_py_dict(py)));
}
```

### `FromPy<T>` and `IntoPy<T>`

Many conversions in PyO3 can't use `std::convert::From` because they need a GIL token.
The [`FromPy`] trait offers an `from_py` method that works just like `from`, except for taking a `Python<'_>` argument.
I.e. `FromPy<T>` could be converting a Rust object into a Python object even though it is called [`FromPy`] - it doesn't say anything about which side of the conversion is a Python object.

Just like `From<T>`, if you implement `FromPy<T>` you gain a blanket implementation of [`IntoPy`] for free.

Eventually, traits such as [`ToPyObject`] will be replaced by this trait and a [`FromPy`] trait will be added that will implement
[`IntoPy`], just like with `From` and `Into`.

[`IntoPy`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.IntoPy.html
[`FromPy`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.FromPy.html
[`FromPyObject`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.FromPyObject.html
[`ToPyObject`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.ToPyObject.html
[`PyObject`]: https://docs.rs/pyo3/latest/pyo3/struct.PyObject.html
[`PyTuple`]: https://docs.rs/pyo3/latest/pyo3/types/struct.PyTuple.html
[`PyAny`]: https://docs.rs/pyo3/latest/pyo3/struct.PyAny.html
[`IntoPyDict`]: https://docs.rs/pyo3/latest/pyo3/types/trait.IntoPyDict.html

[`PyRef`]: https://pyo3.rs/master/doc/pyo3/pycell/struct.PyRef.html
[`PyRefMut`]: https://pyo3.rs/master/doc/pyo3/pycell/struct.PyRefMut.html

[^1]: Requires the `num-complex` optional feature.
