# Python Exception

## Define a new exception

You can use the `create_exception!` macro to define a new exception type:

```rust
use pyo3::import_exception;

create_exception!(module, MyError, pyo3::exceptions::Exception);
```

* `module` is the name of the containing module.
* `MyError` is the name of the new exception type.

For example:

```rust
use pyo3::prelude::*;
use pyo3::create_exception;
use pyo3::types::PyDict;
use pyo3::exceptions::Exception;

create_exception!(mymodule, CustomError, Exception);

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let ctx = PyDict::new(py);

    ctx.set_item("CustomError", py.get_type::<CustomError>()).unwrap();

    py.run("assert str(CustomError) == \"<class 'mymodule.CustomError'>\"", None, Some(&ctx)).unwrap();
    py.run("assert CustomError('oops').args == ('oops',)", None, Some(&ctx)).unwrap();
}
```

## Raise an exception

To raise an exception, first you need to obtain an exception type and construct a new [`PyErr`](https://docs.rs/pyo3/0.2.7/struct.PyErr.html), then call [`PyErr::restore()`](https://docs.rs/pyo3/0.2.7/struct.PyErr.html#method.restore) method to write the exception back to the Python interpreter's global state.

```rust
use pyo3::{Python, PyErr, exc};

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    PyErr::new::<exceptions::TypeError, _>("Error").restore(py);
    assert!(PyErr::occurred(py));
    drop(PyErr::fetch(py));
}
```

If you already have a Python exception instance, you can simply call [`PyErr::from_instance()`](https://docs.rs/pyo3/0.2.7/struct.PyErr.html#method.from_instance).

```rust,ignore
PyErr::from_instance(py, err).restore(py);
```

If rust type exists for exception, then it is possible to use `new` method.
For example each standard exception defined in `exc` module
has corresponding rust type, exceptions defined by `create_exception!` and `import_exception!` macro
have rust type as well.

```rust
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

## Check exception type

Python has an [`isinstance`](https://docs.python.org/3/library/functions.html#isinstance) method to check object type,
in `PyO3` there is a [`Python::is_instance()`](https://docs.rs/pyo3/0.2.7/struct.Python.html#method.is_instance) method which does the same thing.

```rust
use pyo3::{Python, PyBool, PyList};

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    assert!(py.is_instance::<PyBool, _>(PyBool::new(py, true)).unwrap());
    let list = PyList::new(py, &[1, 2, 3, 4]);
    assert!(!py.is_instance::<PyBool, _>(list.as_ref()).unwrap());
    assert!(py.is_instance::<PyList, _>(list.as_ref()).unwrap());
}
```

[`Python::is_instance()`](https://docs.rs/pyo3/0.2.7/struct.Python.html#method.is_instance) calls the underlying [`PyType::is_instance`](https://docs.rs/pyo3/0.2.7/struct.PyType.html#method.is_instance) method to do the actual work.

To check the type of an exception, you can simply do:

```rust
# use pyo3::prelude::*;
# fn main() {
# let gil = Python::acquire_gil();
# let py = gil.python();
# let err = exceptions::TypeError::py_err(NoArgs);
err.is_instance::<exceptions::TypeError>(py);
# }
```

## Handle Rust Error

The vast majority of operations in this library will return [`PyResult<T>`](https://docs.rs/pyo3/0.2.7/type.PyResult.html).
This is an alias for the type `Result<T, PyErr>`.

A [`PyErr`](https://docs.rs/pyo3/0.2.7/struct.PyErr.html) represents a Python exception.
Errors within the `PyO3` library are also exposed as Python exceptions.

PyO3 library handles python exception in two stages. During first stage `PyErr` instance get
created. At this stage python GIL is not required. During second stage, actual python
exception instance get crated and set to python interpreter.

In simple case, for custom errors support implementation of `std::convert::From<T>` trait
for this custom error is enough. `PyErr::new` accepts arguments in form
of `ToPyObject + 'static`. In case if `'static` constraint can not be satisfied or
more complex arguments are required [`PyErrArgument`](https://docs.rs/pyo3/0.2.7/trait.PyErrArguments.html)
trait can be implemented. In that case actual exception arguments creation get delayed
until `Python` object is available.

```rust,ignore
use std::net::TcpListener;
use pyo3::{PyErr, PyResult, exc};

impl std::convert::From<std::io::Error> for PyErr {
    fn from(err: std::io::Error) -> PyErr {
        exceptions::OSError.into()
    }
}

fn connect(s: String) -> PyResult<bool> {
    TcpListener::bind("127.0.0.1:80")?;
    Ok(true)
}
```

The code snippet above will raise `OSError` in Python if `TcpListener::bind()` return an error.

`std::convert::From<T>` trait is implemented for most of the standard library's error
types so `try!` macro or `?` operator can be used.

```rust
use pyo3::prelude::*;

fn parse_int(s: String) -> PyResult<usize> {
    Ok(s.parse::<usize>()?)
}
```

The code snippet above will raise `ValueError` in Python if `String::parse()` return an error.


## Using exceptions defined in python code

It is possible to use exception defined in python code as native rust types.
`import_exception!` macro allows to import specific exception class and defined zst type
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
        Err(_) => Err(UnsupportedOperation::new("not supported: tell")),
        Ok(x) => x.extract::<u64>(py),
    }    
}

```

[`exc`](https://docs.rs/pyo3/0.2.7/exc/index.html) defines exceptions for
several standard library modules.
