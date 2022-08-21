# Python exceptions

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

Python::with_gil(|py| {
    let ctx = [("CustomError", py.get_type::<CustomError>())].into_py_dict(py);
    pyo3::py_run!(py, *ctx, "assert str(CustomError) == \"<class 'mymodule.CustomError'>\"");
    pyo3::py_run!(py, *ctx, "assert CustomError('oops').args == ('oops',)");
});
```

When using PyO3 to create an extension module, you can add the new exception to
the module like this, so that it is importable from Python:

```rust
use pyo3::prelude::*;
use pyo3::types::PyModule;
use pyo3::exceptions::PyException;

pyo3::create_exception!(mymodule, CustomError, PyException);

#[pymodule]
fn mymodule(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    // ... other elements added to module ...
    m.add("CustomError", py.get_type::<CustomError>())?;

    Ok(())
}

```

## Raising an exception

To raise an exception from `pyfunction`s and `pymethods`, you should return an `Err(PyErr)`.
If returned to Python code, this [`PyErr`] will then be raised as a Python exception. Many PyO3 APIs also return [`PyResult`].

If a Rust type exists for the exception, then it is possible to use the `new_err` method.
For example, each standard exception defined in the `pyo3::exceptions` module
has a corresponding Rust type and exceptions defined by [`create_exception!`] and [`import_exception!`] macro have Rust types as well.

```rust
use pyo3::exceptions::PyZeroDivisionError;
use pyo3::prelude::*;

#[pyfunction]
fn divide(a: i32, b: i32) -> PyResult<i32> {
    match a.checked_div(b) {
        Some(q) => Ok(q),
        None => Err(PyZeroDivisionError::new_err("division by zero")),
    }
}
#
# fn main(){
# 	Python::with_gil(|py|{
# 		let fun = pyo3::wrap_pyfunction!(divide, py).unwrap();
# 		fun.call1((1,0)).unwrap_err();
# 		fun.call1((1,1)).unwrap();
# 	});
# }
```

You can manually write and fetch errors in the Python interpreter's global state:

```rust
use pyo3::{Python, PyErr};
use pyo3::exceptions::PyTypeError;

Python::with_gil(|py| {
    PyTypeError::new_err("Error").restore(py);
    assert!(PyErr::occurred(py));
    drop(PyErr::fetch(py));
});
```

If you already have a Python exception object, you can use [`PyErr::from_value`] to create a `PyErr` from it.

## Checking exception types

Python has an [`isinstance`](https://docs.python.org/3/library/functions.html#isinstance) method to check an object's type.
In PyO3 every object has the [`PyAny::is_instance`] and [`PyAny::is_instance_of`] methods which do the same thing.

```rust
use pyo3::Python;
use pyo3::types::{PyBool, PyList};

Python::with_gil(|py| {
    assert!(PyBool::new(py, true).is_instance_of::<PyBool>().unwrap());
    let list = PyList::new(py, &[1, 2, 3, 4]);
    assert!(!list.is_instance_of::<PyBool>().unwrap());
    assert!(list.is_instance_of::<PyList>().unwrap());
});
```

To check the type of an exception, you can similarly do:

```rust
# use pyo3::exceptions::PyTypeError;
# use pyo3::prelude::*;
# Python::with_gil(|py| {
# let err = PyTypeError::new_err(());
err.is_instance_of::<PyTypeError>(py);
# });
```

## Handling Rust errors

The vast majority of operations in this library will return
[`PyResult<T>`]({{#PYO3_DOCS_URL}}/pyo3/prelude/type.PyResult.html),
which is an alias for the type `Result<T, PyErr>`.

A [`PyErr`] represents a Python exception. Errors within the PyO3 library are also exposed as
Python exceptions.

If your code has a custom error type, adding an implementation of `std::convert::From<MyError> for PyErr`
is usually enough. PyO3 will then automatically convert your error to a Python exception when needed.

The following code snippet defines a Rust error named `CustomIOError`. In its `From<CustomIOError> for PyErr`
implementation it returns a `PyErr` representing Python's `OSError`.

```rust
use pyo3::exceptions::PyOSError;
use pyo3::prelude::*;
use std::fmt;

#[derive(Debug)]
struct CustomIOError;

impl std::error::Error for CustomIOError {}

impl fmt::Display for CustomIOError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Oh no!")
    }
}

impl std::convert::From<CustomIOError> for PyErr {
    fn from(err: CustomIOError) -> PyErr {
        PyOSError::new_err(err.to_string())
    }
}

pub struct Connection { /* ... */}

fn bind(addr: String) -> Result<Connection, CustomIOError> {
    if &addr == "0.0.0.0"{
        Err(CustomIOError)
    } else {
        Ok(Connection{ /* ... */})
    }
}

#[pyfunction]
fn connect(s: String) -> Result<(), CustomIOError> {
    bind(s)?;
    Ok(())
}

fn main() {
    Python::with_gil(|py| {
        let fun = pyo3::wrap_pyfunction!(connect, py).unwrap();
        let err = fun.call1(("0.0.0.0",)).unwrap_err();
        assert!(err.is_instance_of::<PyOSError>(py));
    });
}
```

This has been implemented for most of Rust's standard library errors, so that you can use the `?`
("try") operator with them. The following code snippet will raise a `ValueError` in Python if
`String::parse()` returns an error.

```rust
use pyo3::prelude::*;

fn parse_int(s: String) -> PyResult<usize> {
    Ok(s.parse::<usize>()?)
}
#
# use pyo3::exceptions::PyValueError;
#
# fn main() {
#     Python::with_gil(|py| {
#         assert_eq!(parse_int(String::from("1")).unwrap(), 1);
#         assert_eq!(parse_int(String::from("1337")).unwrap(), 1337);
#
#         assert!(parse_int(String::from("-1"))
#             .unwrap_err()
#             .is_instance_of::<PyValueError>(py));
#         assert!(parse_int(String::from("foo"))
#             .unwrap_err()
#             .is_instance_of::<PyValueError>(py));
#         assert!(parse_int(String::from("13.37"))
#             .unwrap_err()
#             .is_instance_of::<PyValueError>(py));
#     })
# }
```

If lazy construction of the Python exception instance is desired, the
[`PyErrArguments`]({{#PYO3_DOCS_URL}}/pyo3/trait.PyErrArguments.html)
trait can be implemented. In that case, actual exception argument creation is delayed
until the `PyErr` is needed.

## Using exceptions defined in Python code

It is possible to use an exception defined in Python code as a native Rust type.
The `import_exception!` macro allows importing a specific exception class and defines a Rust type
for that exception.

```rust
#![allow(dead_code)]
use pyo3::prelude::*;

mod io {
    pyo3::import_exception!(io, UnsupportedOperation);
}

fn tell(file: &PyAny) -> PyResult<u64> {
    match file.call_method0("tell") {
        Err(_) => Err(io::UnsupportedOperation::new_err("not supported: tell")),
        Ok(x) => x.extract::<u64>(),
    }
}

```

[`pyo3::exceptions`]({{#PYO3_DOCS_URL}}/pyo3/exceptions/index.html)
defines exceptions for several standard library modules.

[`create_exception!`]: {{#PYO3_DOCS_URL}}/pyo3/macro.create_exception.html
[`import_exception!`]: {{#PYO3_DOCS_URL}}/pyo3/macro.import_exception.html

[`PyErr`]: {{#PYO3_DOCS_URL}}/pyo3/struct.PyErr.html
[`PyResult`]: {{#PYO3_DOCS_URL}}/pyo3/type.PyResult.html
[`PyErr::from_value`]: {{#PYO3_DOCS_URL}}/pyo3/struct.PyErr.html#method.from_value
[`PyAny::is_instance`]: {{#PYO3_DOCS_URL}}/pyo3/struct.PyAny.html#method.is_instance
[`PyAny::is_instance_of`]: {{#PYO3_DOCS_URL}}/pyo3/struct.PyAny.html#method.is_instance_of
