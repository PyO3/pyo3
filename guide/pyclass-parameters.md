`#[pyclass]` can be used with the following parameters:

|  Parameter  |  Description |
| :-  | :- |
| `constructor` | This is currently only allowed on [variants of complex enums][params-constructor]. It allows customization of the generated class constructor for each variant. It uses the same syntax and supports the same options as the `signature` attribute of functions and methods. |
| <span style="white-space: pre">`crate = "some::path"`</span>  | Path to import the `pyo3` crate, if it's not accessible at `::pyo3`. |
| `dict` | Gives instances of this class an empty `__dict__` to store custom attributes. |
| `eq` | Implements `__eq__` using the `PartialEq` implementation of the underlying Rust datatype. |
| `eq_int` | Implements `__eq__` using `__int__` for simple enums. |
| <span style="white-space: pre">`extends = BaseType`</span>  | Use a custom baseclass. Defaults to [`PyAny`][params-1] |
| <span style="white-space: pre">`freelist = N`</span> |  Implements a [free list][params-2] of size N. This can improve performance for types that are often created and deleted in quick succession. Profile your code to see whether `freelist` is right for you.  |
| <span style="white-space: pre">`frozen`</span> | Declares that your pyclass is immutable. It removes the borrow checker overhead when retrieving a shared reference to the Rust struct, but disables the ability to get a mutable reference. |
| `generic` | Implements runtime parametrization for the class following [PEP 560](https://peps.python.org/pep-0560/). |
| `get_all` | Generates getters for all fields of the pyclass. |
| `hash` | Implements `__hash__` using the `Hash` implementation of the underlying Rust datatype. *Requires `eq` and `frozen`* |
| `immutable_type` | Makes the type object immutable. Supported on 3.14+ with the `abi3` feature active, or 3.10+ otherwise. |
| `mapping` |  Inform PyO3 that this class is a [`Mapping`][params-mapping], and so leave its implementation of sequence C-API slots empty. |
| <span style="white-space: pre">`module = "module_name"`</span> |  Python code will see the class as being defined in this module. Defaults to `builtins`. |
| <span style="white-space: pre">`name = "python_name"`</span> | Sets the name that Python sees this class as. Defaults to the name of the Rust struct. |
| `ord` | Implements `__lt__`, `__gt__`, `__le__`, & `__ge__` using the `PartialOrd` implementation of the underlying Rust datatype. *Requires `eq`* |
| `rename_all = "renaming_rule"` | Applies renaming rules to every getters and setters of a struct, or every variants of an enum. Possible values are: "camelCase", "kebab-case", "lowercase", "PascalCase", "SCREAMING-KEBAB-CASE", "SCREAMING_SNAKE_CASE", "snake_case", "UPPERCASE". |
| `sequence` |  Inform PyO3 that this class is a [`Sequence`][params-sequence], and so leave its C-API mapping length slot empty. |
| `set_all` | Generates setters for all fields of the pyclass. |
| `str` | Implements `__str__` using the `Display` implementation of the underlying Rust datatype or by passing an optional format string `str="<format string>"`. *Note: The optional format string is only allowed for structs.  `name` and `rename_all` are incompatible with the optional format string.  Additional details can be found in the discussion on this [PR](https://github.com/PyO3/pyo3/pull/4233).* |
| `subclass` | Allows other Python classes and `#[pyclass]` to inherit from this class. Enums cannot be subclassed. |
| `unsendable` | Required if your struct is not [`Send`][params-3]. Rather than using `unsendable`, consider implementing your struct in a thread-safe way by e.g. substituting [`Rc`][params-4] with [`Arc`][params-5]. By using `unsendable`, your class will panic when accessed by another thread. Also note the Python's GC is multi-threaded and while unsendable classes will not be traversed on foreign threads to avoid UB, this can lead to memory leaks. |
| `weakref` | Allows this class to be [weakly referenceable][params-6]. |

All of these parameters can either be passed directly on the `#[pyclass(...)]` annotation, or as one or
more accompanying `#[pyo3(...)]` annotations, e.g.:

```rust,ignore
// Argument supplied directly to the `#[pyclass]` annotation.
#[pyclass(name = "SomeName", subclass)]
struct MyClass {}

// Argument supplied as a separate annotation.
#[pyclass]
#[pyo3(name = "SomeName", subclass)]
struct MyClass {}
```

[params-1]: https://docs.rs/pyo3/latest/pyo3/types/struct.PyAny.html
[params-2]: https://en.wikipedia.org/wiki/Free_list
[params-3]: https://doc.rust-lang.org/std/marker/trait.Send.html
[params-4]: https://doc.rust-lang.org/std/rc/struct.Rc.html
[params-5]: https://doc.rust-lang.org/std/sync/struct.Arc.html
[params-6]: https://docs.python.org/3/library/weakref.html
[params-constructor]: https://pyo3.rs/latest/class.html#complex-enums
[params-mapping]: https://pyo3.rs/latest/class/protocols.html#mapping--sequence-types
[params-sequence]: https://pyo3.rs/latest/class/protocols.html#mapping--sequence-types
