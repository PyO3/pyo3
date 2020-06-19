# Appendix A: PyO3 and rust-cpython

PyO3 began as fork of [rust-cpython](https://github.com/dgrunwald/rust-cpython) when rust-cpython wasn't maintained. Over the time PyO3 has become fundamentally different from rust-cpython.

This chapter is based on the discussion in [PyO3/pyo3#55](https://github.com/PyO3/pyo3/issues/55).

## Macros

While rust-cpython has a macro based dsl for declaring modules and classes, PyO3 uses proc macros. PyO3 also doesn't change your struct and functions so you can still use them as normal Rust functions.

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
use pyo3::prelude::*;

#[pyclass]
struct MyClass {
   num: u32,
}

#[pymethods]
impl MyClass {
    #[new]
    fn new(num: u32) -> Self {
        MyClass { num }
    }

    fn half(&self) -> PyResult<u32> {
        Ok(self.num / 2)
    }
}
```

## Ownership and lifetimes

While in rust-cpython you always own python objects, PyO3 allows efficient *borrowed objects*
and most APIs are available with references.

Here is an example of the PyList API:

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

   fn get_item(&self, index: isize) -> &PyAny {...}
}
```

In PyO3, all object references are bounded by the GIL lifetime.
So the owned Python object is not required, and it is safe to have functions like `fn py<'p>(&'p self) -> Python<'p> {}`.

## Error handling

rust-cpython requires a `Python` parameter for constructing a `PyErr`, so error handling ergonomics is pretty bad. It is not possible to use `?` with Rust errors.

PyO3 on other hand does not require `Python` for constructing a `PyErr`, it is only required if you want to raise an exception in Python with the `PyErr::restore()` method. Due to various `std::convert::From<E> for PyErr` implementations for Rust standard error types `E`, propagating `?` is supported automatically.
