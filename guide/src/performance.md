# Performance

To achieve the best possible performance, it is useful to be aware of several tricks and sharp edges concerning PyO3's API.

## `extract` versus `downcast`

Pythonic API implemented using PyO3 are often polymorphic, i.e. they will accept `&PyAny` and try to turn this into multiple more concrete types to which the requested operation is applied. This often leads to chains of calls to `extract`, e.g.

```rust
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::{exceptions::PyTypeError, types::PyList};

fn frobnicate_list(list: &PyList) -> PyResult<&PyAny> {
    todo!()
}

fn frobnicate_vec(vec: Vec<&PyAny>) -> PyResult<&PyAny> {
    todo!()
}

#[pyfunction]
fn frobnicate(value: &PyAny) -> PyResult<&PyAny> {
    if let Ok(list) = value.extract::<&PyList>() {
        frobnicate_list(list)
    } else if let Ok(vec) = value.extract::<Vec<&PyAny>>() {
        frobnicate_vec(vec)
    } else {
        Err(PyTypeError::new_err("Cannot frobnicate that type."))
    }
}
```

This suboptimal as the `FromPyObject<T>` trait requires `extract` to have a `Result<T, PyErr>` return type. For native types like `PyList`, it faster to use `downcast` (which `extract` calls internally) when the error value is ignored. This avoids the costly conversion of a `PyDowncastError` to a `PyErr` required to fulfil the `FromPyObject` contract, i.e.

```rust
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::{exceptions::PyTypeError, types::PyList};
# fn frobnicate_list(list: &PyList) -> PyResult<&PyAny> { todo!() }
# fn frobnicate_vec(vec: Vec<&PyAny>) -> PyResult<&PyAny> { todo!() }
#
#[pyfunction]
fn frobnicate(value: &PyAny) -> PyResult<&PyAny> {
    // Use `downcast` instead of `extract` as turning `PyDowncastError` into `PyErr` is quite costly.
    if let Ok(list) = value.downcast::<PyList>() {
        frobnicate_list(list)
    } else if let Ok(vec) = value.extract::<Vec<&PyAny>>() {
        frobnicate_vec(vec)
    } else {
        Err(PyTypeError::new_err("Cannot frobnicate that type."))
    }
}
```

## Access to GIL-bound reference implies access to GIL token

Calling `Python::with_gil` is effectively a no-op when the GIL is already held, but checking that this is the case still has a cost. If an existing GIL token can not be accessed, for example when implementing a pre-existing trait, but a GIL-bound reference is available, this cost can be avoided by exploiting that access to GIL-bound reference gives zero-cost access to a GIL token via `PyAny::py`.

For example, instead of writing

```rust
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::types::PyList;

struct Foo(PyDetached<PyList>);

struct FooRef<'a>(&'a PyList);

impl PartialEq<Foo> for FooRef<'_> {
    fn eq(&self, other: &Foo) -> bool {
        Python::with_gil(|py| self.0.len() == other.0.as_ref(py).len())
    }
}
```

use more efficient

```rust
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::types::PyList;
# struct Foo(PyDetached<PyList>);
# struct FooRef<'a>(&'a PyList);
#
impl PartialEq<Foo> for FooRef<'_> {
    fn eq(&self, other: &Foo) -> bool {
        // Access to `&'a PyAny` implies access to `Python<'a>`.
        let py = self.0.py();
        self.0.len() == other.0.as_ref(py).len()
    }
}
```
