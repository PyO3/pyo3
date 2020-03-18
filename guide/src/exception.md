# Python Exceptions

## Defining a new exception

You can use the [`create_exception!`] macro to define a new exception type:

```rust
use pyo3::create_exception;

create_exception!(module, MyError, pyo3::exceptions::Exception);
```

* `module` is the name of the containing module.
* `MyError` is the name of the new exception type.

For example:

```rust
use pyo3::prelude::*;
use pyo3::create_exception;
use pyo3::types::IntoPyDict;
use pyo3::exceptions::Exception;

create_exception!(mymodule, CustomError, Exception);

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let ctx = [("CustomError", py.get_type::<CustomError>())].into_py_dict(py);

    py.run("assert str(CustomError) == \"<class 'mymodule.CustomError'>\"", None, Some(&ctx)).unwrap();
    py.run("assert CustomError('oops').args == ('oops',)", None, Some(&ctx)).unwrap();
}
```

## Raising an exception

To raise an exception, first you need to obtain an exception type and construct a new [`PyErr`], then call the [`PyErr::restore`](https://docs.rs/pyo3/latest/pyo3/struct.PyErr.html#method.restore) method to write the exception back to the Python interpreter's global state.

```rust
use pyo3::{Python, PyErr};
use pyo3::exceptions;

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    PyErr::new::<exceptions::TypeError, _>("Error").restore(py);
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

If a Rust type exists for the exception, then it is possible to use the `py_err` method.
For example, each standard exception defined in the `pyo3::exceptions` module
has a corresponding Rust type, exceptions defined by [`create_exception!`] and [`import_exception!`] macro
have Rust types as well.

```rust
# use pyo3::exceptions;
# use pyo3::prelude::*;
# fn check_for_error() -> bool {false}
fn my_func(arg: PyObject) -> PyResult<()> {
    if check_for_error() {
        Err(exceptions::ValueError::py_err("argument is wrong"))
    } else {
        Ok(())
    }
}
```

## Checking exception types

Python has an [`isinstance`](https://docs.python.org/3/library/functions.html#isinstance) method to check an object's type,
in PyO3 there is a [`Python::is_instance`] method which does the same thing.

```rust
use pyo3::Python;
use pyo3::types::{PyBool, PyList};

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    assert!(py.is_instance::<PyBool, _>(PyBool::new(py, true)).unwrap());
    let list = PyList::new(py, &[1, 2, 3, 4]);
    assert!(!py.is_instance::<PyBool, _>(list.as_ref()).unwrap());
    assert!(py.is_instance::<PyList, _>(list.as_ref()).unwrap());
}
```
[`Python::is_instance`] calls the underlying [`PyType::is_instance`](https://docs.rs/pyo3/latest/pyo3/types/struct.PyType.html#method.is_instance)
method to do the actual work.

To check the type of an exception, you can simply do:

```rust
# use pyo3::exceptions;
# use pyo3::prelude::*;
# fn main() {
# let gil = Python::acquire_gil();
# let py = gil.python();
# let err = exceptions::TypeError::py_err(());
err.is_instance::<exceptions::TypeError>(py);
# }
```

## Handling Rust errors

The vast majority of operations in this library will return [`PyResult<T>`](https://docs.rs/pyo3/latest/pyo3/prelude/type.PyResult.html),
which is an alias for the type `Result<T, PyErr>`.

A [`PyErr`] represents a Python exception.
Errors within the PyO3 library are also exposed as Python exceptions.

The PyO3 library handles Python exceptions in two stages. During the first stage, a [`PyErr`] instance is
created. At this stage, holding Python's GIL is not required. During the second stage, an actual Python
exception instance is created and set active in the Python interpreter.

In simple cases, for custom errors adding an implementation of `std::convert::From<T>` trait
for this custom error is enough. `PyErr::new` accepts an argument in the form
of `ToPyObject + 'static`. If the `'static` constraint can not be satisfied or
more complex arguments are required, the
[`PyErrArguments`](https://docs.rs/pyo3/latest/pyo3/trait.PyErrArguments.html)
trait can be implemented. In that case, actual exception argument creation is delayed
until a `Python` object is available.

```rust
# use pyo3::{exceptions, PyErr, PyResult};
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
        exceptions::OSError::py_err(err.to_string())
    }
}

fn connect(s: String) -> PyResult<bool> {
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


## Using exceptions defined in Python code

It is possible to use an exception defined in Python code as a native Rust type.
The `import_exception!` macro allows importing a specific exception class and defines a zero-sized Rust type
for that exception.

```rust
use pyo3::prelude::*;
use pyo3::import_exception;

import_exception!(io, UnsupportedOperation);

fn tell(file: PyObject) -> PyResult<u64> {
    use pyo3::exceptions::*;

    let gil = Python::acquire_gil();
    let py = gil.python();

    match file.call_method0(py, "tell") {
        Err(_) => Err(UnsupportedOperation::py_err("not supported: tell")),
        Ok(x) => x.extract::<u64>(py),
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
