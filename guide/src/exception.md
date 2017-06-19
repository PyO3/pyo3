# Python Exception

## Define a new exception

You can use the `py_exception!` macro to define a new excetpion type:

```rust
py_exception!(module, MyError);
```

* `module` is the name of the containing module.
* `MyError` is the name of the new exception type.

For example:

```rust
#[macro_use] extern crate pyo3;

use pyo3::{Python, PyDict};

py_exception!(mymodule, CustomError);

fn main() {
let gil = Python::acquire_gil();
    let py = gil.python();
    let ctx = PyDict::new(py);

    ctx.set_item(py, "CustomError", py.get_type::<CustomError>()).unwrap();

    py.run("assert str(CustomError) == \"<class 'mymodule.CustomError'>\"", None, Some(&ctx)).unwrap();
    py.run("assert CustomError('oops').args == ('oops',)", None, Some(&ctx)).unwrap();
}
```

## Raise an exception

To raise an exception, first you need to obtain an exception type and construct a new [`PyErr`](https://pyo3.github.io/PyO3/pyo3/struct.PyErr.html), then call [`PyErr::restore()`](https://pyo3.github.io/PyO3/pyo3/struct.PyErr.html#method.restore) method to write the exception back to the Python interpreter's global state.

```rust
extern crate pyo3;

use pyo3::{Python, PyErr, exc};

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    PyErr::new_lazy_init(py.get_type::<exc::TypeError>(), None).restore(py);
    assert!(PyErr::occurred(py));
    drop(PyErr::fetch(py));
}
```

If you already have a Python exception instance, you can simply call [`PyErr::from_instance()`](https://pyo3.github.io/PyO3/pyo3/struct.PyErr.html#method.from_instance).

```rust
PyErr::from_instance(py, err).restore(py);
```

## Check exception type

Python has an [`isinstance`](https://docs.python.org/3/library/functions.html#isinstance) method to check object type,
in `PyO3` there is a [`Python::is_instance()`](https://pyo3.github.io/PyO3/pyo3/struct.Python.html#method.is_instance) method which does the same thing.

```rust
extern crate pyo3;

use pyo3::{Python, PyBool, PyList};

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    assert!(py.is_instance::<PyBool>(py.True().as_ref()).unwrap());
    let list = PyList::new(py, &[1, 2, 3, 4]);
    assert!(!py.is_instance::<PyBool>(list.as_ref()).unwrap());
    assert!(py.is_instance::<PyList>(list.as_ref()).unwrap());
}
```

[`Python::is_instance()`](https://pyo3.github.io/PyO3/pyo3/struct.Python.html#method.is_instance) calls the underlaying [`PyType::is_instance`](https://pyo3.github.io/PyO3/pyo3/struct.PyType.html#method.is_instance) method to do the actual work.

To check the type of an exception, you can simply do:

```rust
let ret = py.is_instance::<exc::TypeError>(&err.instance(py)).expect("Error calling is_instance");
```

## Handle Rust Error

The vast majority of operations in this library will return [`PyResult<T>`](https://pyo3.github.io/PyO3/pyo3/type.PyResult.html).
This is an alias for the type `Result<T, PyErr>`.

A [`PyErr`](https://pyo3.github.io/PyO3/pyo3/struct.PyErr.html) represents a Python exception.
Errors within the `PyO3` library are also exposed as Python exceptions.

The [`ToPyErr`](https://pyo3.github.io/PyO3/pyo3/trait.ToPyErr.html) trait provides a way to convert Rust errors to Python exceptions.

```rust
pub trait ToPyErr {
    fn to_pyerr(&self, _: Python) -> PyErr;
}
```

It's implemented for most of the standard library's error types so that you use [`Result::map_err()`](https://doc.rust-lang.org/std/result/enum.Result.html#method.map_err) to
transform errors to Python exceptions as well as taking advantage of `try!` macro or `?` operator.

```rust
use pyo3::{PyResult, ToPyErr};

fn parse_int(py: Python, s: String) -> PyResult<usize> {
    Ok(s.parse::<usize>().map_err(|e| e.to_pyerr(py))?)
}
```

The code snippet above will raise `ValueError` in Python if `String::parse()` return an error.
