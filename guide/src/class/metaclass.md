# Creating a Metaclass
A [metaclass](https://docs.python.org/3/reference/datamodel.html#metaclasses) is a class that derives `type` and can
be used to influence the construction of other classes.

Some examples of where metaclasses can be used:

- [`ABCMeta`](https://docs.python.org/3/library/abc.html) for defining abstract classes
- [`EnumType`](https://docs.python.org/3/library/enum.html) for defining enums
- [`NamedTuple`](https://docs.python.org/3/library/typing.html#typing.NamedTuple) for defining tuples with elements
  that can be accessed by name in addition to index.
- singleton classes
- automatic registration of classes
- ORM
- serialization / deserialization / validation (e.g. [pydantic](https://docs.pydantic.dev/latest/api/base_model/))

### Example: A Simple Metaclass

```rust
#[pyclass(subclass, extends=PyType)]
#[derive(Default)]
struct MyMetaclass {
    counter: u64,
};

#[pymethods]
impl MyMetaclass {
    #[pyo3(signature = (*_args, **_kwargs))]
    fn __init__(
        slf: Bound<'_, Metaclass>,
        _args: Bound<'_, PyTuple>,
        _kwargs: Option<Bound<'_, PyDict>>,
    ) {
        slf.borrow_mut().counter = 5;
    }

    fn __getitem__(&self, item: u64) -> u64 {
        item + 1
    }

    fn increment_counter(&mut self) {
        self.counter += 1;
    }

    fn get_counter(&self) -> u64 {
        self.counter
    }
}
```

Used like so:
```python
class Foo(metaclass=MyMetaclass):
    def __init__() -> None:
        ...

assert type(Foo) is MyMetaclass
assert Foo.some_var == 123
assert Foo[100] == 101
Foo.increment_counter()
assert Foo.get_counter() == 1
```

In the example above `MyMetaclass` extends `PyType` (making it a metaclass). It does not define `#[new]` as
[this is not supported](https://docs.python.org/3/c-api/type.html#c.PyType_FromMetaclass). Instead `__init__` is
defined which is called whenever a class is created that uses `MyMetaclass` as its metaclass.
The arguments to `__init__` are the same as the arguments to `type(name, bases, kwds)`. A `Default` impl is required
in order to define `__init__`. The data in the struct is initialised to `Default` before `__init__` is called.

When special methods like `__getitem__` are defined for a metaclass they apply to the classes they construct, so
`Foo[123]` calls `MyMetaclass.__getitem__(Foo, 123)`.
