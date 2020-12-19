# Python Exceptions

## Defining a new exception

You can use the [`create_exception!`] macro to define a new exception type:

```rust
use pyo3::create_exception;

create_exception!(module, MyError, pyo3::exceptions::PyException);
```

* `module` is the name of the containing module.
* `MyError` is the name of the new exception type.

For example:

```rust
use pyo3::prelude::*;
use pyo3::create_exception;
use pyo3::types::IntoPyDict;
use pyo3::exceptions::PyException;

create_exception!(mymodule, CustomError, PyException);

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let ctx = [("CustomError", py.get_type::<CustomError>())].into_py_dict(py);

    py.run("assert str(CustomError) == \"<class 'mymodule.CustomError'>\"", None, Some(&ctx)).unwrap();
    py.run("assert CustomError('oops').args == ('oops',)", None, Some(&ctx)).unwrap();
}
```

When using PyO3 to create an extension module, you can add the new exception to
the module like this, so that it is importable from Python:

```rust,ignore

create_exception!(mymodule, CustomError, PyException);

#[pymodule]
fn mymodule(py: Python, m: &PyModule) -> PyResult<()> {
    // ... other elements added to module ...
    m.add("CustomError", py.get_type::<CustomError>())?;

    Ok(())
}

```

## Raising an exception

To raise an exception, first you need to obtain an exception type and construct a new [`PyErr`], then call the [`PyErr::restore`](https://docs.rs/pyo3/latest/pyo3/struct.PyErr.html#method.restore) method to write the exception back to the Python interpreter's global state.

```rust
use pyo3::{Python, PyErr};
use pyo3::exceptions::PyTypeError;

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    PyTypeError::new_err("Error").restore(py);
    assert!(PyErr::occurred(py));
    drop(PyErr::fetch(py));
}
```

From `pyfunction`s and `pyclass` methods, returning an `Err(PyErr)` is enough;
PyO3 will handle restoring the exception on the Python interpreter side.

If you already have a Python exception instance, you can simply call [`PyErr::from_instance`].

```rust,ignore
PyErr::from_instance(py, err).restore(py);
```

If a Rust type exists for the exception, then it is possible to use the `new_err` method.
For example, each standard exception defined in the `pyo3::exceptions` module
has a corresponding Rust type, exceptions defined by [`create_exception!`] and [`import_exception!`] macro
have Rust types as well.

```rust
# use pyo3::exceptions::PyValueError;
# use pyo3::prelude::*;
# fn check_for_error() -> bool {false}
fn my_func(arg: PyObject) -> PyResult<()> {
    if check_for_error() {
        Err(PyValueError::new_err("argument is wrong"))
    } else {
        Ok(())
    }
}
```

## Checking exception types

Python has an [`isinstance`](https://docs.python.org/3/library/functions.html#isinstance) method to check an object's type.
In PyO3 every native type has access to the [`PyAny::is_instance`] method which does the same thing.

```rust
use pyo3::Python;
use pyo3::types::{PyBool, PyList};

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    assert!(PyBool::new(py, true).is_instance::<PyBool>().unwrap());
    let list = PyList::new(py, &[1, 2, 3, 4]);
    assert!(!list.is_instance::<PyBool>().unwrap());
    assert!(list.is_instance::<PyList>().unwrap());
}
```
[`PyAny::is_instance`] calls the underlying [`PyType::is_instance`](https://docs.rs/pyo3/latest/pyo3/types/struct.PyType.html#method.is_instance)
method to do the actual work.

To check the type of an exception, you can similarly do:

```rust
# use pyo3::exceptions::PyTypeError;
# use pyo3::prelude::*;
# let gil = Python::acquire_gil();
# let py = gil.python();
# let err = PyTypeError::new_err(());
err.is_instance::<PyTypeError>(py);
```

## Handling Rust errors

The vast majority of operations in this library will return
[`PyResult<T>`](https://docs.rs/pyo3/latest/pyo3/prelude/type.PyResult.html),
which is an alias for the type `Result<T, PyErr>`.

A [`PyErr`] represents a Python exception. Errors within the PyO3 library are also exposed as
Python exceptions.

If your code has a custom error type e.g. `MyError`, adding an implementation of
`std::convert::From<MyError> for PyErr` is usually enough. PyO3 will then automatically convert
your error to a Python exception when needed.

```rust
# use pyo3::prelude::*;
# use pyo3::exceptions::PyOSError;
# use std::error::Error;
# use std::fmt;
#
# #[derive(Debug)]
# struct CustomIOError;
#
# impl Error for CustomIOError {}
#
# impl fmt::Display for CustomIOError {
#     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
#         write!(f, "Oh no!")
#     }
# }
#
# fn bind(_addr: &str) -> Result<(), CustomIOError> {
#     Err(CustomIOError)
# }
impl std::convert::From<CustomIOError> for PyErr {
    fn from(err: CustomIOError) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
}

#[pyfunction]
fn connect(s: String) -> Result<bool, CustomIOError> {
    bind("127.0.0.1:80")?;
    Ok(true)
}
```

The code snippet above will raise an `OSError` in Python if `bind()` returns a `CustomIOError`.

The `std::convert::From<T>` trait is implemented for most of the Rust standard library's error
types so the `?` operator can be used.

```rust
use pyo3::prelude::*;

fn parse_int(s: String) -> PyResult<usize> {
    Ok(s.parse::<usize>()?)
}
```

The code snippet above will raise a `ValueError` in Python if `String::parse()` returns an error.

If lazy construction of the Python exception instance is desired, the
[`PyErrArguments`](https://docs.rs/pyo3/latest/pyo3/trait.PyErrArguments.html)
trait can be implemented. In that case, actual exception argument creation is delayed
until the `PyErr` is needed.

## Using exceptions defined in Python code

It is possible to use an exception defined in Python code as a native Rust type.
The `import_exception!` macro allows importing a specific exception class and defines a Rust type
for that exception.

```rust
use pyo3::prelude::*;

mod io {
    pyo3::import_exception!(io, UnsupportedOperation);
}

fn tell(file: &PyAny) -> PyResult<u64> {
    use pyo3::exceptions::*;

    match file.call_method0("tell") {
        Err(_) => Err(io::UnsupportedOperation::new_err("not supported: tell")),
        Ok(x) => x.extract::<u64>(),
    }
}

```

[`pyo3::exceptions`](https://docs.rs/pyo3/latest/pyo3/exceptions/index.html)
defines exceptions for several standard library modules.

[`create_exception!`]: https://docs.rs/pyo3/latest/pyo3/macro.create_exception.html
[`import_exception!`]: https://docs.rs/pyo3/latest/pyo3/macro.import_exception.html

[`PyErr`]: https://docs.rs/pyo3/latest/pyo3/struct.PyErr.html
[`PyErr::from_instance`]: https://docs.rs/pyo3/latest/pyo3/struct.PyErr.html#method.from_instance
[`Python::is_instance`]: https://docs.rs/pyo3/latest/pyo3/struct.Python.html#method.is_instance
[`PyAny::is_instance`]: https://docs.rs/pyo3/latest/pyo3/struct.PyAny.html#method.is_instance
