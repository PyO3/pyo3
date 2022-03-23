`#[pyclass]` can be used with the following parameters:

|  Parameter  |  Description |
| :-  | :- |
| <span style="white-space: pre">`crate = "some::path"`</span>  | Path to import the `pyo3` crate, if it's not accessible at `::pyo3`. |
| `dict` | Gives instances of this class an empty `__dict__` to store custom attributes. |
| <span style="white-space: pre">`extends = BaseType`</span>  | Use a custom baseclass. Defaults to [`PyAny`][params-1] |
| <span style="white-space: pre">`freelist = N`</span> |  Implements a [free list][params-2] of size N. This can improve performance for types that are often created and deleted in quick succession. Profile your code to see whether `freelist` is right for you.  |
| <span style="white-space: pre">`module = "module_name"`</span> |  Python code will see the class as being defined in this module. Defaults to `builtins`. |
| <span style="white-space: pre">`name = "python_name"`</span> | Sets the name that Python sees this class as. Defaults to the name of the Rust struct. |
| <span style="white-space: pre">`text_signature = "(arg1, arg2, ...)"`</span> |  Sets the text signature for the Python class' `__new__` method. |
| `subclass` | Allows other Python classes and `#[pyclass]` to inherit from this class. Enums cannot be subclassed. |
| `unsendable` | Required if your struct is not [`Send`][params-3]. Rather than using `unsendable`, consider implementing your struct in a threadsafe way by e.g. substituting [`Rc`][params-4] with [`Arc`][params-5]. By using `unsendable`, your class will panic when accessed by another thread.|
| `weakref` | Allows this class to be [weakly referenceable][params-6]. |

All of these parameters can either be passed directly on the `#[pyclass(...)]` annotation, or as one or
more accompanying `#[pyo3(...)]` annotations, e.g.:

```rust,ignore
// Argument supplied directly to the `#[pyclass]` annotation.
#[pyclass(name = "SomeName", subclass)]
struct MyClass { }

// Argument supplied as a separate annotation.
#[pyclass]
#[pyo3(name = "SomeName", subclass)]
struct MyClass { }
```
