# Performance

To achieve the best possible performance, it is useful to be aware of several tricks and sharp edges concerning PyO3's API.

## `extract` versus `cast`

Pythonic API implemented using PyO3 are often polymorphic, i.e. they will accept `&Bound<'_, PyAny>` and try to turn this into multiple more concrete types to which the requested operation is applied. This often leads to chains of calls to `extract`, e.g.

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::{exceptions::PyTypeError, types::PyList};

fn frobnicate_list<'py>(list: &Bound<'_, PyList>) -> PyResult<Bound<'py, PyAny>> {
    todo!()
}

fn frobnicate_vec<'py>(vec: Vec<Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyAny>> {
    todo!()
}

#[pyfunction]
fn frobnicate<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    if let Ok(list) = value.extract::<Bound<'_, PyList>>() {
        frobnicate_list(&list)
    } else if let Ok(vec) = value.extract::<Vec<Bound<'_, PyAny>>>() {
        frobnicate_vec(vec)
    } else {
        Err(PyTypeError::new_err("Cannot frobnicate that type."))
    }
}
```

This suboptimal as the `FromPyObject<T>` trait requires `extract` to have a `Result<T, PyErr>` return type. For native types like `PyList`, it faster to use `cast` (which `extract` calls internally) when the error value is ignored. This avoids the costly conversion of a `PyDowncastError` to a `PyErr` required to fulfil the `FromPyObject` contract, i.e.

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::{exceptions::PyTypeError, types::PyList};
# fn frobnicate_list<'py>(list: &Bound<'_, PyList>) -> PyResult<Bound<'py, PyAny>> { todo!() }
# fn frobnicate_vec<'py>(vec: Vec<Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyAny>> { todo!() }
#
#[pyfunction]
fn frobnicate<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    // Use `cast` instead of `extract` as turning `PyDowncastError` into `PyErr` is quite costly.
    if let Ok(list) = value.cast::<PyList>() {
        frobnicate_list(list)
    } else if let Ok(vec) = value.extract::<Vec<Bound<'_, PyAny>>>() {
        frobnicate_vec(vec)
    } else {
        Err(PyTypeError::new_err("Cannot frobnicate that type."))
    }
}
```

## Access to Bound implies access to Python token

Calling `Python::attach` is effectively a no-op when we're already attached to the interpreter, but checking that this is the case still has a cost. If an existing Python token can not be accessed, for example when implementing a pre-existing trait, but a Python-bound reference is available, this cost can be avoided by exploiting that access to Python-bound reference gives zero-cost access to a Python token via `Bound::py`.

For example, instead of writing

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::types::PyList;

struct Foo(Py<PyList>);

struct FooBound<'py>(Bound<'py, PyList>);

impl PartialEq<Foo> for FooBound<'_> {
    fn eq(&self, other: &Foo) -> bool {
        Python::attach(|py| {
            let len = other.0.bind(py).len();
            self.0.len() == len
        })
    }
}
```

use the more efficient

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# struct Foo(Py<PyList>);
# struct FooBound<'py>(Bound<'py, PyList>);
#
impl PartialEq<Foo> for FooBound<'_> {
    fn eq(&self, other: &Foo) -> bool {
        // Access to `&Bound<'py, PyAny>` implies access to `Python<'py>`.
        let py = self.0.py();
        let len = other.0.bind(py).len();
        self.0.len() == len
    }
}
```

## Calling Python callables (`__call__`)
CPython support multiple calling protocols: [`tp_call`] and [`vectorcall`]. [`vectorcall`] is a more efficient protocol unlocking faster calls.
PyO3 will try to dispatch Python `call`s using the [`vectorcall`] calling convention to archive maximum performance if possible and falling back to [`tp_call`] otherwise.
This is implemented using the (internal) `PyCallArgs` trait. It defines how Rust types can be used as Python `call` arguments. This trait is currently implemented for
- Rust tuples, where each member implements `IntoPyObject`,
- `Bound<'_, PyTuple>`
- `Py<PyTuple>`
Rust tuples may make use of [`vectorcall`] where as `Bound<'_, PyTuple>` and `Py<PyTuple>` can only use [`tp_call`]. For maximum performance prefer using Rust tuples as arguments.


[`tp_call`]: https://docs.python.org/3/c-api/call.html#the-tp-call-protocol
[`vectorcall`]: https://docs.python.org/3/c-api/call.html#the-vectorcall-protocol

## Disable the global reference pool

PyO3 uses global mutable state to keep track of deferred reference count updates implied by `impl<T> Drop for Py<T>` being called without being attached to the interpreter. The necessary synchronization to obtain and apply these reference count updates when PyO3-based code next attaches to the interpreter is somewhat expensive and can become a significant part of the cost of crossing the Python-Rust boundary.

This functionality can be avoided by setting the `pyo3_disable_reference_pool` conditional compilation flag. This removes the global reference pool and the associated costs completely. However, it does _not_ remove the `Drop` implementation for `Py<T>` which is necessary to interoperate with existing Rust code written without PyO3-based code in mind. To stay compatible with the wider Rust ecosystem in these cases, we keep the implementation but abort when `Drop` is called without being attached to the interpreter. If `pyo3_leak_on_drop_without_reference_pool` is additionally enabled, objects dropped without being attached to Python will be leaked instead which is always sound but might have determinal effects like resource exhaustion in the long term.

This limitation is important to keep in mind when this setting is used, especially when embedding Python code into a Rust application as it is quite easy to accidentally drop a `Py<T>` (or types containing it like `PyErr`, `PyBackedStr` or `PyBackedBytes`) returned from `Python::attach` without making sure to re-attach beforehand. For example, the following code

```rust,ignore
# use pyo3::prelude::*;
# use pyo3::types::PyList;
let numbers: Py<PyList> = Python::attach(|py| PyList::empty(py).unbind());

Python::attach(|py| {
    numbers.bind(py).append(23).unwrap();
});

Python::attach(|py| {
    numbers.bind(py).append(42).unwrap();
});
```

will abort if the list not explicitly disposed via

```rust
# use pyo3::prelude::*;
# use pyo3::types::PyList;
let numbers: Py<PyList> = Python::attach(|py| PyList::empty(py).unbind());

Python::attach(|py| {
    numbers.bind(py).append(23).unwrap();
});

Python::attach(|py| {
    numbers.bind(py).append(42).unwrap();
});

Python::attach(move |py| {
    drop(numbers);
});
```

[conditional-compilation]: https://doc.rust-lang.org/reference/conditional-compilation.html
