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
| `dict[K, V]`  | `HashMap<K, V>`, `BTreeMap<K, V>`, `hashbrown::HashMap<K, V>`[^2] | `&PyDict` |
| `tuple[T, U]` | `(T, U)`, `Vec<T>`              | `&PyTuple`           |
| `set[T]`      | `HashSet<T>`, `BTreeSet<T>`, `hashbrown::HashSet<T>`[^2] | `&PySet` |
| `frozenset[T]` | `HashSet<T>`, `BTreeSet<T>`, `hashbrown::HashSet<T>`[^2] | `&PyFrozenSet` |
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
| `typing.Union[...]` | See [`#[derive(FromPyObject)]`](#deriving-a-hrefhttpsdocsrspyo3latestpyo3conversiontraitfrompyobjecthtmlfrompyobjecta-for-enums) | - |

There are also a few special types related to the GIL and Rust-defined `#[pyclass]`es which may come in useful:

| What          | Description |
| ------------- | ------------------------------------- |
| `Python`      | A GIL token, used to pass to PyO3 constructors to prove ownership of the GIL |
| `Py<T>`       | A Python object isolated from the GIL lifetime. This can be sent to other threads. |
| `PyObject`    | An alias for `Py<PyAny>`              |
| `&PyCell<T>`  | A `#[pyclass]` value owned by Python. |
| `PyRef<T>`    | A `#[pyclass]` borrowed immutably.    |
| `PyRefMut<T>` | A `#[pyclass]` borrowed mutably.      |

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

Because these types are references, in some situations the Rust compiler may ask for lifetime annotations. If this is the case, you should use `Py<PyAny>`, `Py<PyDict>` etc. instead - which are also zero-cost. For all of these Python-native types `T`, `Py<T>` can be created from `T` with an `.into()` conversion.

If your function is fallible, it should return `PyResult<T>` or `Result<T, E>` where `E` implements `From<E> for PyErr`. This will raise a `Python` exception if the `Err` variant is returned.

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

#### Deriving [`FromPyObject`]

[`FromPyObject`] can be automatically derived for many kinds of structs and enums
if the member types themselves implement `FromPyObject`. This even includes members
with a generic type `T: FromPyObject`. Derivation for empty enums, enum variants and
structs is not supported.

#### Deriving [`FromPyObject`] for structs

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyStruct {
    my_string: String,
}
```

The derivation generates code that will per default access the attribute `my_string` on
the Python object, i.e. `obj.getattr("my_string")`, and call `extract()` on the attribute.
It is also possible to access the value on the Python object through `obj.get_item("my_string")`
by setting the attribute `pyo3(item)` on the field:
```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyStruct {
    #[pyo3(item)]
    my_string: String,
}
```

The argument passed to `getattr` and `get_item` can also be configured:

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyStruct {
    #[pyo3(item("key"))]
    string_in_mapping: String,
    #[pyo3(attribute("name"))]
    string_attr: String,
}
```

This tries to extract `string_attr` from the attribute `name` and `string_in_mapping`
from a mapping with the key `"key"`. The arguments for `attribute` are restricted to
non-empty string literals while `item` can take any valid literal that implements
`ToBorrowedObject`.

#### Deriving [`FromPyObject`] for tuple structs

Tuple structs are also supported but do not allow customizing the extraction. The input is
always assumed to be a Python tuple with the same length as the Rust type, the `n`th field
is extracted from the `n`th item in the Python tuple.

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyTuple(String, String);
```

Tuple structs with a single field are treated as wrapper types which are described in the
following section. To override this behaviour and ensure that the input is in fact a tuple,
specify the struct as
```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyTuple((String,));
```

#### Deriving [`FromPyObject`] for wrapper types

The `pyo3(transparent)` attribute can be used on structs with exactly one field. This results
in extracting directly from the input object, i.e. `obj.extract()`, rather than trying to access
an item or attribute. This behaviour is enabled per default for newtype structs and tuple-variants
with a single field.

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
struct RustyTransparentTupleStruct(String);

#[derive(FromPyObject)]
#[pyo3(transparent)]
struct RustyTransparentStruct {
    inner: String,
}
```

#### Deriving [`FromPyObject`] for enums

The `FromPyObject` derivation for enums generates code that tries to extract the variants in the
order of the fields. As soon as a variant can be extracted succesfully, that variant is returned.
This makes it possible to extract Python types like `Union[str, int]`.

The same customizations and restrictions described for struct derivations apply to enum variants,
i.e. a tuple variant assumes that the input is a Python tuple, and a struct variant defaults to
extracting fields as attributes but can be configured in the same manner. The `transparent`
attribute can be applied to single-field-variants.

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
enum RustyEnum<'a> {
    Int(usize), // input is a positive int
    String(String), // input is a string
    IntTuple(usize, usize), // input is a 2-tuple with positive ints
    StringIntTuple(String, usize), // input is a 2-tuple with String and int
    Coordinates3d { // needs to be in front of 2d
        x: usize,
        y: usize,
        z: usize,
    },
    Coordinates2d { // only gets checked if the input did not have `z`
        #[pyo3(attribute("x"))]
        a: usize,
        #[pyo3(attribute("y"))]
        b: usize,
    },
    #[pyo3(transparent)]
    CatchAll(&'a PyAny), // This extraction never fails
}
```

If none of the enum variants match, a `PyValueError` containing the names of the
tested variants is returned. The names reported in the error message can be customized
through the `pyo3(annotation = "name")` attribute, e.g. to use conventional Python type
names:

```
use pyo3::prelude::*;

#[derive(FromPyObject)]
enum RustyEnum {
    #[pyo3(transparent, annotation = "str")]
    String(String),
    #[pyo3(transparent, annotation = "int")]
    Int(isize),
}
```

If the input is neither a string nor an integer, the error message will be:
`"Can't convert <INPUT> to Union[str, int]"`, where `<INPUT>` is replaced by the type name and
`repr()` of the input object.

#### `#[derive(FromPyObject)]` Container Attributes
- `pyo3(transparent)`
    - extract the field directly from the object as `obj.extract()` instead of `get_item()` or
      `getattr()`
    - Newtype structs and tuple-variants are treated as transparent per default.
    - only supported for single-field structs and enum variants
- `pyo3(annotation = "name")`
    - changes the name of the failed variant in the generated error message in case of failure.
    - e.g. `pyo3("int")` reports the variant's type as `int`.
    - only supported for enum variants

#### `#[derive(FromPyObject)]` Field Attributes
- `pyo3(attribute)`, `pyo3(attribute("name"))`
    - retrieve the field from an attribute, possibly with a custom name specified as an argument
    - argument must be a string-literal.
- `pyo3(item)`, `pyo3(item("key"))`
    - retrieve the field from a mapping, possibly with the custom key specified as an argument.
    - can be any literal that implements `ToBorrowedObject`

### `IntoPy<T>`

This trait defines the to-python conversion for a Rust type. It is usually implemented as
`IntoPy<PyObject>`, which is the trait needed for returning a value from `#[pyfunction]` and
`#[pymethods]`.

All types in PyO3 implement this trait, as does a `#[pyclass]` which doesn't use `extends`.

Occasionally you may choose to implement this for custom types which are mapped to Python types
_without_ having a unique python type.

```
use pyo3::prelude::*;

struct MyPyObjectWrapper(PyObject);

impl IntoPy<PyObject> for MyPyObjectWrapper {
    fn into_py(self, py: Python) -> PyObject {
        self.0
    }
}
```

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

[`IntoPy`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.IntoPy.html
[`FromPyObject`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.FromPyObject.html
[`ToPyObject`]: https://docs.rs/pyo3/latest/pyo3/conversion/trait.ToPyObject.html
[`PyObject`]: https://docs.rs/pyo3/latest/pyo3/type.PyObject.html
[`PyTuple`]: https://docs.rs/pyo3/latest/pyo3/types/struct.PyTuple.html
[`PyAny`]: https://docs.rs/pyo3/latest/pyo3/struct.PyAny.html
[`IntoPyDict`]: https://docs.rs/pyo3/latest/pyo3/types/trait.IntoPyDict.html

[`PyRef`]: https://docs.rs/pyo3/latest/pyo3/pycell/struct.PyRef.html
[`PyRefMut`]: https://docs.rs/pyo3/latest/pyo3/pycell/struct.PyRefMut.html

[^1]: Requires the `num-complex` optional feature.
[^2]: Requires the `hashbrown` optional feature.
