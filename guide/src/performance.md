# Performance

To achieve the best possible performance, it is useful to be aware of several tricks and sharp edges concerning PyO3's API.

## `extract` versus `downcast`

Pythonic API implemented using PyO3 are often polymorphic, i.e. they will accept `&Bound<'_, PyAny>` and try to turn this into multiple more concrete types to which the requested operation is applied. This often leads to chains of calls to `extract`, e.g.

```rust
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

This suboptimal as the `FromPyObject<T>` trait requires `extract` to have a `Result<T, PyErr>` return type. For native types like `PyList`, it faster to use `downcast` (which `extract` calls internally) when the error value is ignored. This avoids the costly conversion of a `PyDowncastError` to a `PyErr` required to fulfil the `FromPyObject` contract, i.e.

```rust
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::{exceptions::PyTypeError, types::PyList};
# fn frobnicate_list<'py>(list: &Bound<'_, PyList>) -> PyResult<Bound<'py, PyAny>> { todo!() }
# fn frobnicate_vec<'py>(vec: Vec<Bound<'py, PyAny>>) -> PyResult<Bound<'py, PyAny>> { todo!() }
#
#[pyfunction]
fn frobnicate<'py>(value: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
    // Use `downcast` instead of `extract` as turning `PyDowncastError` into `PyErr` is quite costly.
    if let Ok(list) = value.downcast::<PyList>() {
        frobnicate_list(list)
    } else if let Ok(vec) = value.extract::<Vec<Bound<'_, PyAny>>>() {
        frobnicate_vec(vec)
    } else {
        Err(PyTypeError::new_err("Cannot frobnicate that type."))
    }
}
```

## Access to Bound implies access to GIL token

Calling `Python::with_gil` is effectively a no-op when the GIL is already held, but checking that this is the case still has a cost. If an existing GIL token can not be accessed, for example when implementing a pre-existing trait, but a GIL-bound reference is available, this cost can be avoided by exploiting that access to GIL-bound reference gives zero-cost access to a GIL token via `Bound::py`.

For example, instead of writing

```rust
# #![allow(dead_code)]
# use pyo3::prelude::*;
# use pyo3::types::PyList;

struct Foo(Py<PyList>);

struct FooBound<'py>(Bound<'py, PyList>);

impl PartialEq<Foo> for FooBound<'_> {
    fn eq(&self, other: &Foo) -> bool {
        Python::with_gil(|py| {
            let len = other.0.bind(py).len();
            self.0.len() == len
        })
    }
}
```

use the more efficient

```rust
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
