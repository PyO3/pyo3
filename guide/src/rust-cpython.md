# Appendix: pyo3 and rust-cpython

Pyo3 began as fork of [rust-cpython](https://github.com/dgrunwald/rust-cpython) when rust-cpython wasn't maintained. Over the time pyo3 has become fundamentally different from rust-cpython.

This chapter is based on the discussion in [PyO3/pyo3#55](https://github.com/PyO3/pyo3/issues/55).

## Macros

While rust-cpython has a macro based dsl for declaring modules and classes, pyo3 use proc macros and spezialization. Pyo3 also doesn't change your struct and functions so you can still use them as normal rust functions. The disadvantage is that proc macros and spezialization currently only work on nightly.

**rust-cpython**

```rust,ignore
py_class!(class MyClass |py| {
    data number: i32;
    def __new__(_cls, arg: i32) -> PyResult<MyClass> {
        MyClass::create_instance(py, arg)
    }
    def half(&self) -> PyResult<i32> {
        Ok(self.number(py) / 2)
    }
});
```

**pyo3**

```rust
#![feature(specialization)]

extern crate pyo3;

use pyo3::prelude::*;
use pyo3::PyRawObject;

#[pyclass]
struct MyClass {
   num: u32,
}

#[pymethods]
impl MyClass {
    #[new]
    fn __new__(obj: &PyRawObject, num: u32) -> PyResult<()> {
        obj.init(|| {
            MyClass {
                num,
            }
        })
    }

    fn half(&self) -> PyResult<u32> {
        Ok(self.num / 2)
    }
}
```

## Ownership and lifetimes

All objects are owned by pyo3 library and all apis available with references, while in rust-cpython, you own python objects.

Here is example of PyList api:

**rust-cpython**

```rust,ignore
impl PyList {

   fn new(py: Python) -> PyList {...}

   fn get_item(&self, py: Python, index: isize) -> PyObject {...}
}
```

**pyo3**

```rust,ignore
impl PyList {

   fn new(py: Python) -> &PyList {...}

   fn get_item(&self, index: isize) -> &PyObject {...}
}
```

Because pyo3 allows only references to python object, all reference have the Gil lifetime. So the python object is not required, and it is safe to have functions like `fn py<'p>(&'p self) -> Python<'p> {}`.

## Error handling

rust-cpython requires a `Python` parameter for `PyErr`, so error handling ergonomics is pretty bad. It is not possible to use `?` with rust errors.

`pyo3` on other hand does not require `Python` for `PyErr`, it is only required if you want to raise an exception in python with the `PyErr::restore()` method. Due to the `std::convert::From<Err> for PyErr` trait `?` is supported automatically.
