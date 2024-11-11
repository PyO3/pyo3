# `#[pyclass]` thread safety

Python objects are freely shared between threads by the Python interpreter. This means that:
- there is no control which thread might eventually drop the `#[pyclass]` object, meaning `Send` is required.
- multiple threads can potentially be reading the `#[pyclass]` data simultaneously, meaning `Sync` is required.

This section of the guide discusses various datastructures which can be used to make types satisfy these requirements.

In special cases where it is known that your Python application is never going to use threads (this is rare!), these thread-safety requirements can be opted-out with [`#[pyclass(unsendable)]`](../class.md#customizing-the-class), at the cost of making concurrent access to the Rust data be runtime errors. This is only for very specific use cases; it is almost always better to make proper thread-safe types.

## Making `#[pyclass]` types thread-safe

The general challenge with thread-safety is to make sure that two threads cannot produce a data race, i.e. unsynchronized writes to the same data at the same time. A data race produces an unpredictable result and is forbidden by Rust.

By default, `#[pyclass]` employs an ["interior mutability" pattern](../class.md#bound-and-interior-mutability) to allow for either multiple `&T` references or a single exclusive `&mut T` reference to access the data. This allows for simple `#[pyclass]` types to be thread-safe automatically, at the cost of runtime checking for concurrent access. Errors will be raised if the usage overlaps.

For example, the below simple class is thread-safe:

```rust
# use pyo3::prelude::*;

#[pyclass]
struct MyClass {
    x: i32,
    y: i32,
}

#[pymethods]
impl MyClass {
    fn get_x(&self) -> i32 {
        self.x
    }

    fn set_y(&mut self, value: i32) -> i32 {
        self.y = value;
    }
}
```

In the above example, if calls to `get_x` and `set_y` overlap (from two different threads) then at least one of those threads will experience a runtime error indicating that the data was "already borrowed".

There are three main ways that more complicated thread-safety topics can become relevant when writing `#[pyclass]` types:
  - To avoid possible "already borrowed" runtime errors, a `#[pyclass]` may choose to use [atomic data structures](https://doc.rust-lang.org/std/sync/atomic/).
  - To avoid possible "already borrowed" runtime errors, a `#[pyclass]` may choose to use locks.
  - If a `#[pyclass]` contains data which is itself not `Sync` or `Send`, then it becomes the responsibility of the `#[pyclass]` type to be a safe wrapper around the unsynchronized data.

The following sections touch on each of these options

## Using atomic datastructures

## Using locks

## Wrapping unsynchronized data
