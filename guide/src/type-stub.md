# Type stub generation (`*.pyi` files) and introspection

*This feature is still in active development.*
*See [the related issue](https://github.com/PyO3/pyo3/issues/5137).*

*For documentation on type stubs and how to use them with stable PyO3, refer to [this page](python-typing-hints.md)*

PyO3 has a work in progress support to generate [type stub files](https://typing.python.org/en/latest/spec/distributing.html#stub-files).

It works using:

1. PyO3 macros (`#[pyclass]`) that generate constant JSON strings that are then included in the built binaries by rustc if the `experimental-inspect` feature is enabled.
2. The `pyo3-introspection` crate that can parse the generated binaries, extract the JSON strings and build stub files from it.
3. Build tools like `maturin` exposing options in their CLI API to generate the stubs file.

For example, the following Rust code

```rust
#[pymodule]
pub mod example {
    use pyo3::prelude::*;

    #[pymodule_export]
    pub const CONSTANT: &str = "FOO";

    #[pyclass(eq)]
    #[derive(Eq)]
    struct Class {
        value: usize
    }

    #[pymethods]
    impl Class {
        #[new]
        fn new(value: usize) -> Self {
            Self { value }
        }

        #[getter]
        fn value(&self) -> usize {
            self.value
        }
    }

    #[pyfunction]
    #[pyo3(signature = (arg: "list[int]") -> "list[int]")]
    fn list_of_int_identity(arg: Bound<'_, PyAny>) -> Bound<'_, PyAny> {
        arg
    }
}
```

will generate the following stub file:

```python
import typing

__all__ = ["CONSTANT", "Class", "list_of_int_identity"]

CONSTANT: typing.Final = "FOO"


class Class:
    def __init__(self, value: typing.SupportsIndex) -> None: ...

    @property
    def value(self) -> int: ...

    def __eq__(self, value: object, /) -> bool: ...
    def __ne__(self, value: object, /) -> bool: ...


def list_of_int_identity(arg: "list[int]") -> "list[int]": ...
```

The only piece of new syntax is that the `#[pyo3(signature = ...)]` attribute can contain type annotations like `#[pyo3(signature = (arg: "list[int]") -> "list[int]")]` (note the `""` around type annotations).
This is useful when PyO3 is not able to derive proper type annotations by itself.

To generate stubs file with `maturin` you can use `maturin generate-stubs --output stubs` that will build the project then generate the stubs in the `stubs` directory.
You can also directly integrate the stubs in the built wheels by doing `maturin build --generate-stubs` (works also with `maturin develop`).

PyO3 also provides the smaller `pyo3-introspection` binary that allows to generate stubs from an existing built extension using something like `pyo3-introspection my_module_binary.so my_module_name output` to introspect the `my_module_binary.so` dynamic library for the `my_module_name` Python module and generate its stubs in the `output` directory.

## Protocols

`pyprotocol!` declares a [`typing.Protocol`](https://typing.python.org/en/latest/spec/protocol.html) for objects a `FromPyObject` implementation accepts by their methods.
Like `FromPyObject::INPUT_TYPE`, it is only available with the `experimental-inspect` feature.
It also works in library crates:

```rust,ignore
pyo3::pyprotocol! {
    /// A binary stream that can be read from and written to
    pub SupportsReadWrite {
        #[pyo3(signature = (n = -1, /))]
        fn read(&self, n: isize) -> Py<PyBytes>;
        fn write(&self, data: &[u8]) -> usize;
    }
}

impl<'py> FromPyObject<'_, 'py> for ReadWriteFile<'py> {
    const INPUT_TYPE: PyStaticExpr = SupportsReadWrite::TYPE_HINT;
    // ...
}
```

A `#[pyfunction]` taking a `ReadWriteFile` is then annotated with `SupportsReadWrite`, which is added to the stubs of the module that uses it:

```python
@type_check_only
class SupportsReadWrite(Protocol):
    """
    A binary stream that can be read from and written to
    """

    def read(self, n: int = ..., /) -> bytes: ...
    def write(self, data: bytes, /) -> SupportsIndex: ...
```

- The methods describe calls made from Rust, so argument types use `IntoPyObject::OUTPUT_TYPE` and return types `FromPyObject::INPUT_TYPE`.
  Parameters are positional-only unless a `#[pyo3(signature = (...))]` is given, so that implementations can name them freely.
- A protocol only appears in the stubs if a type hint refers to it.
  It is declared in the deepest module containing all the modules that use it.
  If that module already has a member with the same name, the protocol is renamed to e.g. `_SupportsReadWrite2`; `#[pyo3(name = "...")]` sets another name.
- It only exists in the stubs, hence [`@type_check_only`](https://docs.python.org/3/library/typing.html#typing.type_check_only) and it is not listed in `__all__`: it can be imported in an `if TYPE_CHECKING:` block, but not at runtime.

## Constraints and limitations

- The `experimental-inspect` feature is required to generate the introspection fragments.
- Lots of features are not implemented yet.
  See [the related issue](https://github.com/PyO3/pyo3/issues/5137) for a list of them.
- Introspection only works with Python modules declared with an inline Rust module.
  Modules declared using a function are not supported.
- `FromPyObject::INPUT_TYPE` and `IntoPyObject::OUTPUT_TYPE` must be implemented for PyO3 to get the proper input/output type annotations to use.
- The generated stubs depend on the Python version the extension is built for.
  For example, the `disjoint_base` decorator is imported from `typing` when targeting Python 3.15 or newer and from `typing_extensions` otherwise.
  If you commit generated stubs to your repository, generate them with the oldest Python version you support (or with the matching `abi3-pyXY` feature) so that they are valid for every version.
- PyO3 is not able to introspect the content of `#[pymodule]` and `#[pymodule_init]` functions.
  If they are present, the module is tagged as incomplete using a fake `def __getattr__(name: str) -> Incomplete: ...` function [following best practices](https://typing.python.org/en/latest/guides/writing_stubs.html#incomplete-stubs).
  Such a module also gets no `__all__` declaration, because the set of names it exports at runtime is not known.
  A `#[pymodule_init]` function [declared without the module argument](module.md#procedural-initialization) is exempt: it is not handed the module, so the module is taken to be complete and keeps its `__all__`.
  A `Python<'_>` marker on its own does not count as the module argument.
