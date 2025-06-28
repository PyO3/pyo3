# Error handling

This chapter contains a little background of error handling in Rust and how PyO3 integrates this with Python exceptions.

This covers enough detail to create a `#[pyfunction]` which raises Python exceptions from errors originating in Rust.

There is a later section of the guide on [Python exceptions](../exception.md) which covers exception types in more detail.

## Representing Python exceptions

Rust code uses the generic [`Result<T, E>`] enum to propagate errors. The error type `E` is chosen by the code author to describe the possible errors which can happen.

PyO3 has the [`PyErr`] type which represents a Python exception. If a PyO3 API could result in a Python exception being raised, the return type of that `API` will be [`PyResult<T>`], which is an alias for the type `Result<T, PyErr>`.

In summary:
- When Python exceptions are raised and caught by PyO3, the exception will be stored in the `Err` variant of the `PyResult`.
- Passing Python exceptions through Rust code then uses all the "normal" techniques such as the `?` operator, with `PyErr` as the error type.
- Finally, when a `PyResult` crosses from Rust back to Python via PyO3, if the result is an `Err` variant the contained exception will be raised.

(There are many great tutorials on Rust error handling and the `?` operator, so this guide will not go into detail on Rust-specific topics.)

## Raising an exception from a function

As indicated in the previous section, when a `PyResult` containing an `Err` crosses from Rust to Python, PyO3 will raise the exception contained within.

Accordingly, to raise an exception from a `#[pyfunction]`, change the return type `T` to `PyResult<T>`. When the function returns an `Err` it will raise a Python exception. (Other `Result<T, E>` types can be used as long as the error `E` has a `From` conversion for `PyErr`, see [custom Rust error types](#custom-rust-error-types) below.)

This also works for functions in `#[pymethods]`.

For example, the following `check_positive` function raises a `ValueError` when the input is negative:

```rust
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

#[pyfunction]
fn check_positive(x: i32) -> PyResult<()> {
    if x < 0 {
        Err(PyValueError::new_err("x is negative"))
    } else {
        Ok(())
    }
}
#
# fn main(){
# 	Python::attach(|py|{
# 		let fun = pyo3::wrap_pyfunction!(check_positive, py).unwrap();
# 		fun.call1((-1,)).unwrap_err();
# 		fun.call1((1,)).unwrap();
# 	});
# }
```

All built-in Python exception types are defined in the [`pyo3::exceptions`] module. They have a `new_err` constructor to directly build a `PyErr`, as seen in the example above.

## Custom Rust error types

PyO3 will automatically convert a `Result<T, E>` returned by a `#[pyfunction]` into a `PyResult<T>` as long as there is an implementation of `std::from::From<E> for PyErr`. Many error types in the Rust standard library have a [`From`] conversion defined in this way.

If the type `E` you are handling is defined in a third-party crate, see the section on [foreign rust error types](#foreign-rust-error-types) below for ways to work with this error.

The following example makes use of the implementation of `From<ParseIntError> for PyErr` to raise exceptions encountered when parsing strings as integers:

```rust
# use pyo3::prelude::*;
use std::num::ParseIntError;

#[pyfunction]
fn parse_int(x: &str) -> Result<usize, ParseIntError> {
    x.parse()
}

# fn main() {
#     Python::attach(|py| {
#         let fun = pyo3::wrap_pyfunction!(parse_int, py).unwrap();
#         let value: usize = fun.call1(("5",)).unwrap().extract().unwrap();
#         assert_eq!(value, 5);
#     });
# }
```

When passed a string which doesn't contain a floating-point number, the exception raised will look like the below:

```python
>>> parse_int("bar")
Traceback (most recent call last):
  File "<stdin>", line 1, in <module>
ValueError: invalid digit found in string
```

As a more complete example, the following snippet defines a Rust error named `CustomIOError`. It then defines a `From<CustomIOError> for PyErr`, which returns a `PyErr` representing Python's `OSError`.
Therefore, it can use this error in the result of a `#[pyfunction]` directly, relying on the conversion if it has to be propagated into a Python exception.

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

pub struct Connection {/* ... */}

fn bind(addr: String) -> Result<Connection, CustomIOError> {
    if &addr == "0.0.0.0" {
        Err(CustomIOError)
    } else {
        Ok(Connection{ /* ... */})
    }
}

#[pyfunction]
fn connect(s: String) -> Result<(), CustomIOError> {
    bind(s)?;
    // etc.
    Ok(())
}

fn main() {
    Python::attach(|py| {
        let fun = pyo3::wrap_pyfunction!(connect, py).unwrap();
        let err = fun.call1(("0.0.0.0",)).unwrap_err();
        assert!(err.is_instance_of::<PyOSError>(py));
    });
}
```

If lazy construction of the Python exception instance is desired, the
[`PyErrArguments`]({{#PYO3_DOCS_URL}}/pyo3/trait.PyErrArguments.html)
trait can be implemented instead of `From`. In that case, actual exception argument creation is delayed
until the `PyErr` is needed.

A final note is that any errors `E` which have a `From` conversion can be used with the `?`
("try") operator with them. An alternative implementation of the above `parse_int` which instead returns `PyResult` is below:

```rust
use pyo3::prelude::*;

fn parse_int(s: String) -> PyResult<usize> {
    let x = s.parse()?;
    Ok(x)
}
#
# use pyo3::exceptions::PyValueError;
#
# fn main() {
#     Python::attach(|py| {
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

## Foreign Rust error types

The Rust compiler will not permit implementation of traits for types outside of the crate where the type is defined. (This is known as the "orphan rule".)

Given a type `OtherError` which is defined in third-party code, there are two main strategies available to integrate it with PyO3:

- Create a newtype wrapper, e.g. `MyOtherError`. Then implement `From<MyOtherError> for PyErr` (or `PyErrArguments`), as well as `From<OtherError>` for `MyOtherError`.
- Use Rust's Result combinators such as `map_err` to write code freely to convert `OtherError` into whatever is needed. This requires boilerplate at every usage however gives unlimited flexibility.

To detail the newtype strategy a little further, the key trick is to return `Result<T, MyOtherError>` from the `#[pyfunction]`. This means that PyO3 will make use of `From<MyOtherError> for PyErr` to create Python exceptions while the `#[pyfunction]` implementation can use `?` to convert `OtherError` to `MyOtherError` automatically.

The following example demonstrates this for some imaginary third-party crate `some_crate` with a function `get_x` returning `Result<i32, OtherError>`:

```rust
# mod some_crate {
#   pub struct OtherError(());
#   impl OtherError {
#       pub fn message(&self) -> &'static str { "some error occurred" }
#   }
#   pub fn get_x() -> Result<i32, OtherError> { Ok(5) }
# }

use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use some_crate::{OtherError, get_x};

struct MyOtherError(OtherError);

impl From<MyOtherError> for PyErr {
    fn from(error: MyOtherError) -> Self {
        PyValueError::new_err(error.0.message())
    }
}

impl From<OtherError> for MyOtherError {
    fn from(other: OtherError) -> Self {
        Self(other)
    }
}

#[pyfunction]
fn wrapped_get_x() -> Result<i32, MyOtherError> {
    // get_x is a function returning Result<i32, OtherError>
    let x: i32 = get_x()?;
    Ok(x)
}

# fn main() {
#     Python::attach(|py| {
#         let fun = pyo3::wrap_pyfunction!(wrapped_get_x, py).unwrap();
#         let value: usize = fun.call0().unwrap().extract().unwrap();
#         assert_eq!(value, 5);
#     });
# }
```


[`From`]: https://doc.rust-lang.org/stable/std/convert/trait.From.html
[`Result<T, E>`]: https://doc.rust-lang.org/stable/std/result/enum.Result.html
[`PyResult<T>`]: {{#PYO3_DOCS_URL}}/pyo3/prelude/type.PyResult.html
[`PyErr`]: {{#PYO3_DOCS_URL}}/pyo3/struct.PyErr.html
[`pyo3::exceptions`]: {{#PYO3_DOCS_URL}}/pyo3/exceptions/index.html
