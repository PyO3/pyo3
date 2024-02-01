## Mapping of Rust types to Python types

When writing functions callable from Python (such as a `#[pyfunction]` or in a `#[pymethods]` block), the trait `FromPyObject` is required for function arguments, and `IntoPy<PyObject>` is required for function return values.

Consult the tables in the following section to find the Rust types provided by PyO3 which implement these traits.

### Argument Types

When accepting a function argument, it is possible to either use Rust library types or PyO3's Python-native types. (See the next section for discussion on when to use each.)

The table below contains the Python type and the corresponding function argument types that will accept them:

| Python        | Rust                            | Rust (Python-native) |
| ------------- |:-------------------------------:|:--------------------:|
| `object`      | -                               | `&PyAny`             |
| `str`         | `String`, `Cow<str>`, `&str`, `char`, `OsString`, `PathBuf`, `Path` | `&PyString`, `&PyUnicode` |
| `bytes`       | `Vec<u8>`, `&[u8]`, `Cow<[u8]>` | `&PyBytes`           |
| `bool`        | `bool`                          | `&PyBool`            |
| `int`         | `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `i64`, `u64`, `i128`, `u128`, `isize`, `usize`, `num_bigint::BigInt`[^1], `num_bigint::BigUint`[^1] | `&PyLong` |
| `float`       | `f32`, `f64`                    | `&PyFloat`           |
| `complex`     | `num_complex::Complex`[^2]      | `&PyComplex`         |
| `list[T]`     | `Vec<T>`                        | `&PyList`            |
| `dict[K, V]`  | `HashMap<K, V>`, `BTreeMap<K, V>`, `hashbrown::HashMap<K, V>`[^3], `indexmap::IndexMap<K, V>`[^4] | `&PyDict` |
| `tuple[T, U]` | `(T, U)`, `Vec<T>`              | `&PyTuple`           |
| `set[T]`      | `HashSet<T>`, `BTreeSet<T>`, `hashbrown::HashSet<T>`[^3] | `&PySet` |
| `frozenset[T]` | `HashSet<T>`, `BTreeSet<T>`, `hashbrown::HashSet<T>`[^3] | `&PyFrozenSet` |
| `bytearray`   | `Vec<u8>`, `Cow<[u8]>`          | `&PyByteArray`       |
| `slice`       | -                               | `&PySlice`           |
| `type`        | -                               | `&PyType`            |
| `module`      | -                               | `&PyModule`          |
| `collections.abc.Buffer` | -                    | `PyBuffer<T>`        |
| `datetime.datetime` | `SystemTime`              | `&PyDateTime`        |
| `datetime.date` | -                             | `&PyDate`            |
| `datetime.time` | -                             | `&PyTime`            |
| `datetime.tzinfo` | -                           | `&PyTzInfo`          |
| `datetime.timedelta` | `Duration`               | `&PyDelta`           |
| `decimal.Decimal` | `rust_decimal::Decimal`[^5] | -                    |
| `ipaddress.IPv4Address` | `std::net::IpAddr`, `std::net::IpV4Addr` | - |
| `ipaddress.IPv6Address` | `std::net::IpAddr`, `std::net::IpV6Addr` | - |
| `os.PathLike ` | `PathBuf`, `Path`              | `&PyString`, `&PyUnicode` |
| `pathlib.Path` | `PathBuf`, `Path`              | `&PyString`, `&PyUnicode` |
| `typing.Optional[T]` | `Option<T>`              | -                    |
| `typing.Sequence[T]` | `Vec<T>`                 | `&PySequence`        |
| `typing.Mapping[K, V]` | `HashMap<K, V>`, `BTreeMap<K, V>`, `hashbrown::HashMap<K, V>`[^3], `indexmap::IndexMap<K, V>`[^4] | `&PyMapping` |
| `typing.Iterator[Any]` | -                      | `&PyIterator`        |
| `typing.Union[...]` | See [`#[derive(FromPyObject)]`](traits.html#deriving-a-hrefhttpsdocsrspyo3latestpyo3conversiontraitfrompyobjecthtmlfrompyobjecta-for-enums) | - |

There are also a few special types related to the GIL and Rust-defined `#[pyclass]`es which may come in useful:

| What          | Description |
| ------------- | ------------------------------------- |
| `Python`      | A GIL token, used to pass to PyO3 constructors to prove ownership of the GIL |
| `Py<T>`       | A Python object isolated from the GIL lifetime. This can be sent to other threads. |
| `PyObject`    | An alias for `Py<PyAny>`              |
| `&PyCell<T>`  | A `#[pyclass]` value owned by Python. |
| `PyRef<T>`    | A `#[pyclass]` borrowed immutably.    |
| `PyRefMut<T>` | A `#[pyclass]` borrowed mutably.      |

For more detail on accepting `#[pyclass]` values as function arguments, see [the section of this guide on Python Classes](../class.md).

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
| `Cow<[u8]>`   | `bytes`                         |
| `HashMap<K, V>` | `Dict[K, V]`                  |
| `BTreeMap<K, V>` | `Dict[K, V]`                 |
| `HashSet<T>`  | `Set[T]`                        |
| `BTreeSet<T>` | `Set[T]`                        |
| `&PyCell<T: PyClass>` | `T`                     |
| `PyRef<T: PyClass>` | `T`                       |
| `PyRefMut<T: PyClass>` | `T`                    |

[^1]: Requires the `num-bigint` optional feature.

[^2]: Requires the `num-complex` optional feature.

[^3]: Requires the `hashbrown` optional feature.

[^4]: Requires the `indexmap` optional feature.

[^5]: Requires the `rust_decimal` optional feature.
