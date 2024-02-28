# Python functions

The `#[pyfunction]` attribute is used to define a Python function from a Rust function. Once defined, the function needs to be added to a [module](./module.md) using the `wrap_pyfunction!` macro.

The following example defines a function called `double` in a Python module called `my_extension`:

```rust
use pyo3::prelude::*;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule]
fn my_extension(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)?;
    Ok(())
}
```

This chapter of the guide explains full usage of the `#[pyfunction]` attribute. In this first section, the following topics are covered:

- [Function options](#function-options)
  - [`#[pyo3(name = "...")]`](#name)
  - [`#[pyo3(signature = (...))]`](#signature)
  - [`#[pyo3(text_signature = "...")]`](#text_signature)
  - [`#[pyo3(pass_module)]`](#pass_module)
- [Per-argument options](#per-argument-options)
- [Advanced function patterns](#advanced-function-patterns)
- [`#[pyfn]` shorthand](#pyfn-shorthand)

There are also additional sections on the following topics:

- [Function Signatures](./function/signature.md)

## Function options

The `#[pyo3]` attribute can be used to modify properties of the generated Python function. It can take any combination of the following options:

  - <a name="name"></a> `#[pyo3(name = "...")]`

    Overrides the name exposed to Python.

    In the following example, the Rust function `no_args_py` will be added to the Python module
    `module_with_functions` as the Python function `no_args`:

    ```rust
    use pyo3::prelude::*;

    #[pyfunction]
    #[pyo3(name = "no_args")]
    fn no_args_py() -> usize {
        42
    }

    #[pymodule]
    fn module_with_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(no_args_py, m)?)?;
        Ok(())
    }

    # Python::with_gil(|py| {
    #     let m = pyo3::wrap_pymodule!(module_with_functions)(py);
    #     assert!(m.getattr(py, "no_args").is_ok());
    #     assert!(m.getattr(py, "no_args_py").is_err());
    # });
    ```

  - <a name="signature"></a> `#[pyo3(signature = (...))]`

    Defines the function signature in Python. See [Function Signatures](./function/signature.md).

  - <a name="text_signature"></a> `#[pyo3(text_signature = "...")]`

    Overrides the PyO3-generated function signature visible in Python tooling (such as via [`inspect.signature`]). See the [corresponding topic in the Function Signatures subchapter](./function/signature.md#making-the-function-signature-available-to-python).

  - <a name="pass_module" ></a> `#[pyo3(pass_module)]`

    Set this option to make PyO3 pass the containing module as the first argument to the function. It is then possible to use the module in the function body. The first argument **must** be of type `&PyModule`.

    The following example creates a function `pyfunction_with_module` which returns the containing module's name (i.e. `module_with_fn`):

    ```rust
    use pyo3::prelude::*;
    use pyo3::types::PyString;

    #[pyfunction]
    #[pyo3(pass_module)]
    fn pyfunction_with_module<'py>(module: &Bound<'py, PyModule>) -> PyResult<Bound<'py, PyString>> {
        module.name()
    }

    #[pymodule]
    fn module_with_fn(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(pyfunction_with_module, m)?)
    }
    ```

## Per-argument options

The `#[pyo3]` attribute can be used on individual arguments to modify properties of them in the generated function. It can take any combination of the following options:

  - <a name="from_py_with"></a> `#[pyo3(from_py_with = "...")]`

    Set this on an option to specify a custom function to convert the function argument from Python to the desired Rust type, instead of using the default `FromPyObject` extraction. The function signature must be `fn(&PyAny) -> PyResult<T>` where `T` is the Rust type of the argument.

    The following example uses `from_py_with` to convert the input Python object to its length:

    ```rust
    use pyo3::prelude::*;

    fn get_length(obj: &PyAny) -> PyResult<usize> {
        let length = obj.len()?;
        Ok(length)
    }

    #[pyfunction]
    fn object_length(#[pyo3(from_py_with = "get_length")] argument: usize) -> usize {
        argument
    }

    # Python::with_gil(|py| {
    #     let f = pyo3::wrap_pyfunction!(object_length)(py).unwrap();
    #     assert_eq!(f.call1((vec![1, 2, 3],)).unwrap().extract::<usize>().unwrap(), 3);
    # });
    ```

## Advanced function patterns

### Calling Python functions in Rust

You can pass Python `def`'d functions and built-in functions to Rust functions [`PyFunction`]
corresponds to regular Python functions while [`PyCFunction`] describes built-ins such as
`repr()`.

You can also use [`PyAny::is_callable`] to check if you have a callable object. `is_callable` will
return `true` for functions (including lambdas), methods and objects with a `__call__` method.
You can call the object with [`PyAny::call`] with the args as first parameter and the kwargs
(or `None`) as second parameter. There are also [`PyAny::call0`] with no args and [`PyAny::call1`]
with only positional args.

### Calling Rust functions in Python

The ways to convert a Rust function into a Python object vary depending on the function:

- Named functions, e.g. `fn foo()`: add `#[pyfunction]` and then use [`wrap_pyfunction!`] to get the corresponding [`PyCFunction`].
- Anonymous functions (or closures), e.g. `foo: fn()` either:
  - use a `#[pyclass]` struct which stores the function as a field and implement `__call__` to call the stored function.
  - use `PyCFunction::new_closure` to create an object directly from the function.

[`PyAny::is_callable`]: {{#PYO3_DOCS_URL}}/pyo3/struct.PyAny.html#tymethod.is_callable
[`PyAny::call`]: {{#PYO3_DOCS_URL}}/pyo3/struct.PyAny.html#tymethod.call
[`PyAny::call0`]: {{#PYO3_DOCS_URL}}/pyo3/struct.PyAny.html#tymethod.call0
[`PyAny::call1`]: {{#PYO3_DOCS_URL}}/pyo3/struct.PyAny.html#tymethod.call1
[`PyObject`]: {{#PYO3_DOCS_URL}}/pyo3/type.PyObject.html
[`wrap_pyfunction!`]: {{#PYO3_DOCS_URL}}/pyo3/macro.wrap_pyfunction.html
[`PyFunction`]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyFunction.html
[`PyCFunction`]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyCFunction.html

### Accessing the FFI functions

In order to make Rust functions callable from Python, PyO3 generates an `extern "C"`
function whose exact signature depends on the Rust signature.  (PyO3 chooses the optimal
Python argument passing convention.) It then embeds the call to the Rust function inside this
FFI-wrapper function. This wrapper handles extraction of the regular arguments and the keyword
arguments from the input `PyObject`s.

The `wrap_pyfunction` macro can be used to directly get a `Bound<PyCFunction>` given a
`#[pyfunction]` and a `PyModule`: `wrap_pyfunction!(rust_fun, module)`.

## `#[pyfn]` shorthand

There is a shorthand to `#[pyfunction]` and `wrap_pymodule!`: the function can be placed inside the module definition and
annotated with `#[pyfn]`. To simplify PyO3, it is expected that `#[pyfn]` may be removed in a future release (See [#694](https://github.com/PyO3/pyo3/issues/694)).

An example of `#[pyfn]` is below:

```rust
use pyo3::prelude::*;

#[pymodule]
fn my_extension(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    #[pyfn(m)]
    fn double(x: usize) -> usize {
        x * 2
    }

    Ok(())
}
```

`#[pyfn(m)]` is just syntactic sugar for `#[pyfunction]`, and takes all the same options
documented in the rest of this chapter. The code above is expanded to the following:

```rust
use pyo3::prelude::*;

#[pymodule]
fn my_extension(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[pyfunction]
    fn double(x: usize) -> usize {
        x * 2
    }

    m.add_function(wrap_pyfunction!(double, m)?)?;
    Ok(())
}
```

[`inspect.signature`]: https://docs.python.org/3/library/inspect.html#inspect.signature
