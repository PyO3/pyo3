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
