# Typing and IDE hints for your Python package

PyO3 provides an easy to use interface to code native Python libraries in Rust. The accompanying Maturin allows you to build and publish them as a package. Yet, for a better user experience, Python libraries should provide typing hints and documentation for all public entities, so that IDEs can show them during development and type analyzing tools such as `mypy` can use them to properly verify the code.

Currently the best solution for the problem is to manually maintain `*.pyi` files and ship them along with the package.

PyO3 is working on automated their generation. See the [type stub generation](type-stub.md) documentation for a description of the current state of automated generation.

## Introduction to `pyi` files

`pyi` files (an abbreviation for `Python Interface`) are called "stub files" in most of the documentation related to them. A very good definition of what it is can be found in [old MyPy documentation](https://github.com/python/mypy/wiki/Creating-Stubs-For-Python-Modules):

> A stubs file only contains a description of the public interface of the module without any implementations.

There is also [extensive documentation on type stubs on the official Python typing documentation](https://typing.readthedocs.io/en/latest/source/stubs.html).

Most Python developers probably already encountered them when trying to use their IDE's "Go to Definition" function on any builtin type. For example, the definitions of a few standard exceptions look like this:

```python
class BaseException(object):
    args: Tuple[Any, ...]
    __cause__: BaseException | None
    __context__: BaseException | None
    __suppress_context__: bool
    __traceback__: TracebackType | None
    def __init__(self, *args: object) -> None: ...
    def __str__(self) -> str: ...
    def __repr__(self) -> str: ...
    def with_traceback(self: _TBE, tb: TracebackType | None) -> _TBE: ...

class SystemExit(BaseException):
    code: int

class Exception(BaseException): ...

class StopIteration(Exception):
    value: Any
```

As we can see, those are not full definitions containing implementation, but just a description of the interface. It is usually all that the user of the library needs.

### What do the PEPs say?

At the time of writing this documentation, the `pyi` files are referenced in four PEPs.

[PEP8 - Style Guide for Python Code - #Function Annotations](https://www.python.org/dev/peps/pep-0008/#function-annotations) (last point) recommends all third party library creators to provide stub files as the source of knowledge about the package for type checker tools.

> (...) it is expected that users of third party library packages may want to run type checkers over those packages. For this purpose [PEP 484](https://www.python.org/dev/peps/pep-0484) recommends the use of stub files: .pyi files that are read by the type checker in preference of the corresponding .py files. (...)

[PEP484 - Type Hints - #Stub Files](https://www.python.org/dev/peps/pep-0484/#stub-files) defines stub files as follows.

> Stub files are files containing type hints that are only for use by the type checker, not at runtime.

It contains a specification for them (highly recommended reading, since it contains at least one thing that is not used in normal Python code) and also some general information about where to store the stub files.

[PEP561 - Distributing and Packaging Type Information](https://www.python.org/dev/peps/pep-0561/) describes in detail how to build packages that will enable type checking. In particular it contains information about how the stub files must be distributed in order for type checkers to use them.

[PEP560 - Core support for typing module and generic types](https://www.python.org/dev/peps/pep-0560/) describes the details on how Python's type system internally supports generics, including both runtime behavior and integration with static type checkers.

## How to do it?

[PEP561](https://www.python.org/dev/peps/pep-0561/) recognizes three ways of distributing type information:

* `inline` - the typing is placed directly in source (`py`) files;
* `separate package with stub files` - the typing is placed in `pyi` files distributed in their own, separate package;
* `in-package stub files` - the typing is placed in `pyi` files distributed in the same package as source files.

The first way is tricky with PyO3 since we do not have `py` files. When it has been investigated and necessary changes are implemented, this document will be updated.

The second way is easy to do, and the whole work can be fully separated from the main library code. The example repo for the package with stub files can be found in [PEP561 references section](https://www.python.org/dev/peps/pep-0561/#references): [Stub package repository](https://github.com/ethanhs/stub-package)

The third way is described below.

### Including `pyi` files in your PyO3/Maturin build package

When source files are in the same package as stub files, they should be placed next to each other. We need a way to do that with Maturin. Also, in order to mark our package as typing-enabled we need to add an empty file named `py.typed` to the package.

#### If you do not have other Python files

If you do not need to add any other Python files apart from `pyi` to the package, Maturin provides a way to do most of the work for you. As documented in the [Maturin Guide](https://github.com/PyO3/maturin/#mixed-rustpython-projects), the only thing you need to do is to create a stub file for your module named `<module_name>.pyi` in your project root and Maturin will do the rest.

```text
my-rust-project/
├── Cargo.toml
├── my_project.pyi  # <<< add type stubs for Rust functions in the my_project module here
├── pyproject.toml
└── src
    └── lib.rs
```

For an example `pyi` file see the [`my_project.pyi` content](#my_projectpyi-content) section.

#### If you need other Python files

If you need to add other Python files apart from `pyi` to the package, you can do it also, but that requires some more work. Maturin provides an easy way to add files to a package ([documentation](https://github.com/PyO3/maturin/blob/0dee40510083c03607834c821eea76964140a126/Readme.md#mixed-rustpython-projects)). You just need to create a folder with the name of your module next to the `Cargo.toml` file (for customization see documentation linked above).

The folder structure would be:

```text
my-project
├── Cargo.toml
├── my_project
│   ├── __init__.py
│   ├── my_project.pyi
│   ├── other_python_file.py
│   └── py.typed
├── pyproject.toml
├── Readme.md
└── src
    └── lib.rs
```

Let's go a little bit more into detail regarding the files inside the package folder.

##### `__init__.py` content

As we now specify our own package content, we have to provide the `__init__.py` file, so the folder is treated as a package and we can import things from it. We can always use the same content that Maturin creates for us if we do not specify a Python source folder. For PyO3 bindings it would be:

```python
from .my_project import *
```

That way everything that is exposed by our native module can be imported directly from the package.

##### `py.typed` requirement

As stated in [PEP561](https://www.python.org/dev/peps/pep-0561/):
> Package maintainers who wish to support type checking of their code MUST add a marker file named py.typed to their package supporting typing. This marker applies recursively: if a top-level package includes it, all its sub-packages MUST support type checking as well.

If we do not include that file, some IDEs might still use our `pyi` files to show hints, but the type checkers might not. MyPy will raise an error in this situation:

```text
error: Skipping analyzing "my_project": found module but no type hints or library stubs
```

The file is just a marker file, so it should be empty.

##### `my_project.pyi` content

Our module stub file. This document does not aim at describing how to write them, since you can find a lot of documentation on it, starting from the already quoted [PEP484](https://www.python.org/dev/peps/pep-0484/#stub-files).

The example can look like this:

```python
class Car:
    """
    A class representing a car.

    :param body_type: the name of body type, e.g. hatchback, sedan
    :param horsepower: power of the engine in horsepower
    """
    def __init__(self, body_type: str, horsepower: int) -> None: ...

    @classmethod
    def from_unique_name(cls, name: str) -> 'Car':
        """
        Creates a Car based on unique name

        :param name: model name of a car to be created
        :return: a Car instance with default data
        """

    def best_color(self) -> str:
        """
        Gets the best color for the car.

        :return: the name of the color our great algorithm thinks is the best for this car
        """
```

### Supporting Generics

Type annotations can also be made generic in Python. They are useful for working
with different types while maintaining type safety. Usually, generic classes
inherit from the `typing.Generic` metaclass.

Take for example the following `.pyi` file that specifies a `Car` that can
accept multiple types of wheels:

```python
from typing import Generic, TypeVar

W = TypeVar('W')

class Car(Generic[W]):
    def __init__(self, wheels: list[W]) -> None: ...

    def get_wheels(self) -> list[W]: ...

    def change_wheel(self, wheel_number: int, wheel: W) -> None: ...
```

This way, the end-user can specify the type with variables such as `truck: Car[SteelWheel] = ...`
and `f1_car: Car[AlloyWheel] = ...`.

There is also a special syntax for specifying generic types in Python 3.12+:

```python
class Car[W]:
    def __init__(self, wheels: list[W]) -> None: ...

    def get_wheels(self) -> list[W]: ...
```

#### Runtime Behaviour

Stub files (`pyi`) are only useful for static type checkers and ignored at runtime. Therefore,
PyO3 classes do not inherit from `typing.Generic` even if specified in the stub files.

This can cause some runtime issues, as annotating a variable like `f1_car: Car[AlloyWheel] = ...`
can make Python call magic methods that are not defined. 

To overcome this limitation, implementers can pass the `generic` parameter to `pyclass` in Rust:

```rust ignore
#[pyclass(generic)]
```

#### Advanced Users

`#[pyclass(generic)]` implements a very simple runtime behavior that accepts
any generic argument. Advanced users can opt to manually implement
[`__class_geitem__`](https://docs.python.org/3/reference/datamodel.html#emulating-generic-types)
for the generic class to have more control.

```rust ignore
impl MyClass {
    #[classmethod]
    #[pyo3(signature = (key, /))]
    pub fn __class_getitem__(
        cls: &Bound<'_, PyType>,
        key: &Bound<'_, PyAny>,
    ) -> PyResult<Py<PyAny>> {
        /* implementation details */
    }
}
```

Note that [`pyo3::types::PyGenericAlias`][pygenericalias] can be helfpul when implementing
`__class_geitem__` as it can create [`types.GenericAlias`][genericalias] objects from Rust.

[pygenericalias]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.pygenericalias
[genericalias]: https://docs.python.org/3/library/types.html#types.GenericAlias