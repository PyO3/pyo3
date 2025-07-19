# Python exceptions

## Defining a new exception

Use the [`create_exception!`] macro:

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

# fn main() -> PyResult<()> {
Python::attach(|py| {
    let ctx = [("CustomError", py.get_type::<CustomError>())].into_py_dict(py)?;
    pyo3::py_run!(
        py,
        *ctx,
        "assert str(CustomError) == \"<class 'mymodule.CustomError'>\""
    );
    pyo3::py_run!(py, *ctx, "assert CustomError('oops').args == ('oops',)");
#   Ok(())
})
# }
```

When using PyO3 to create an extension module, you can add the new exception to
the module like this, so that it is importable from Python:

```rust,no_run
# fn main() {}
use pyo3::prelude::*;
use pyo3::exceptions::PyException;

pyo3::create_exception!(mymodule, CustomError, PyException);

#[pymodule]
mod mymodule {
    #[pymodule_export]
    use super::CustomError;

    // ... other elements added to module ...
}
```

## Raising an exception

As described in the [function error handling](./function/error-handling.md) chapter, to raise an exception from a `#[pyfunction]` or `#[pymethods]`, return an `Err(PyErr)`. PyO3 will automatically raise this exception for you when returning the result to Python.

You can also manually write and fetch errors in the Python interpreter's global state:

```rust
use pyo3::{Python, PyErr};
use pyo3::exceptions::PyTypeError;

Python::attach(|py| {
    PyTypeError::new_err("Error").restore(py);
    assert!(PyErr::occurred(py));
    drop(PyErr::fetch(py));
});
```

## Checking exception types

Python has an [`isinstance`](https://docs.python.org/3/library/functions.html#isinstance) method to check an object's type.
In PyO3 every object has the [`PyAny::is_instance`] and [`PyAny::is_instance_of`] methods which do the same thing.

```rust,no_run
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyList};

# fn main() -> PyResult<()> {
Python::attach(|py| {
    assert!(PyBool::new(py, true).is_instance_of::<PyBool>());
    let list = PyList::new(py, &[1, 2, 3, 4])?;
    assert!(!list.is_instance_of::<PyBool>());
    assert!(list.is_instance_of::<PyList>());
# Ok(())
})
# }
```

To check the type of an exception, you can similarly do:

```rust,no_run
# use pyo3::exceptions::PyTypeError;
# use pyo3::prelude::*;
# Python::attach(|py| {
# let err = PyTypeError::new_err(());
err.is_instance_of::<PyTypeError>(py);
# });
```

## Using exceptions defined in Python code

It is possible to use an exception defined in Python code as a native Rust type.
The `import_exception!` macro allows importing a specific exception class and defines a Rust type
for that exception.

```rust,no_run
#![allow(dead_code)]
use pyo3::prelude::*;

mod io {
    pyo3::import_exception!(io, UnsupportedOperation);
}

fn tell(file: &Bound<'_, PyAny>) -> PyResult<u64> {
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
[`PyAny::is_instance`]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.is_instance
[`PyAny::is_instance_of`]: {{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.is_instance_of

## Creating more complex exceptions

If you need to create an exception with more complex behavior, you can also manually create a subclass of `PyException`:

```rust
#![allow(dead_code)]
# #[cfg(any(not(feature = "abi3")))] {
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use pyo3::exceptions::PyException;

#[pyclass(extends=PyException)]
struct CustomError {
    #[pyo3(get)]
    url: String,

    #[pyo3(get)]
    message: String,
}

#[pymethods]
impl CustomError {
    #[new]
    fn new(url: String, message: String) -> Self {
        Self { url, message }
    }
}

# fn main() -> PyResult<()> {
Python::attach(|py| {
    let ctx = [("CustomError", py.get_type::<CustomError>())].into_py_dict(py)?;
    pyo3::py_run!(
        py,
        *ctx,
        "assert str(CustomError) == \"<class 'builtins.CustomError'>\", repr(CustomError)"
    );
    pyo3::py_run!(py, *ctx, "assert CustomError('https://example.com', 'something went bad').args == ('https://example.com', 'something went bad')");
    pyo3::py_run!(py, *ctx, "assert CustomError('https://example.com', 'something went bad').url == 'https://example.com'");
#   Ok(())
})
# }
# }

```

Note that this is not possible when the ``abi3`` feature is enabled, as that prevents subclassing ``PyException``.
