# Type stub generation (`*.pyi` files) and introspection

*This feature is still in active development. See [the related issue](https://github.com/PyO3/pyo3/issues/5137).*

*For documentation on type stubs and how to use them with stable PyO3, refer to [this page](python-typing-hints.md)*

PyO3 has a work in progress support to generate [type stub files](https://typing.python.org/en/latest/spec/distributing.html#stub-files).

It works using:
1. PyO3 macros (`#[pyclass]`) that generate constant JSON strings that are then included in the built binaries by rustc if the `experimental-inspect` feature is enabled.
2. The `pyo3-introspection` crate that can parse the generated binaries, extract the JSON strings and build stub files from it.
3. [Not done yet] Build tools like `maturin` exposing `pyo3-introspection` features in their CLI API.

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

CONSTANT: typing.Final = "FOO"

class Class:
    def __init__(self, value: int) -> None: ...

    @property
    def value(self) -> int: ...

    def __eq__(self, other: Class) -> bool: ...
    def __ne__(self, other: Class) -> bool: ...

def list_of_int_identity(arg: list[int]) -> list[int]: ...
```

The only piece of added syntax is that the `#[pyo3(signature = ...)]` attribute
can now contain type annotations like `#[pyo3(signature = (arg: "list[int]") -> "list[int]")]`
(note the `""` around type annotations).
This is useful when PyO3 is not able to derive proper type annotations by itself.

## Constraints and limitations

- The `experimental-inspect` feature is required to generate the introspection fragments.
- Lots of features are not implemented yet. See [the related issue](https://github.com/PyO3/pyo3/issues/5137) for a list of them.
- Introspection only works with Python modules declared with an inline Rust module. Modules declared using a function are not supported.
- `FromPyObject::INPUT_TYPE` and `IntoPyObject::OUTPUT_TYPE` must be implemented for PyO3 to get the proper input/output type annotations to use.
- Because `FromPyObject::INPUT_TYPE` and `IntoPyObject::OUTPUT_TYPE` are `const` it is not possible to build yet smart generic annotations for containers like `concat!("list[", T::OUTPUT_TYPE, "]")`. See [this tracking issue](https://github.com/rust-lang/rust/issues/76560).
- PyO3 is not able to introspect the content of `#[pymodule]` and `#[pymodule_init]` functions. If they are present, the module is tagged as incomplete using a fake `def __getattr__(name: str) -> Incomplete: ...` function [following best practices](https://typing.python.org/en/latest/guides/writing_stubs.html#incomplete-stubs).
