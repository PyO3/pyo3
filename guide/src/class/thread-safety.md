# `#[pyclass]` thread safety

Python objects are freely shared between threads by the Python interpreter. This means that:
- there is no control which thread might eventually drop the `#[pyclass]` object, meaning `Send` is required.
- multiple threads can potentially be reading the `#[pyclass]` data simultaneously, meaning `Sync` is required.

This section of the guide discusses various data structures which can be used to make types satisfy these requirements.

In special cases where it is known that your Python application is never going to use threads (this is rare!), these thread-safety requirements can be opted-out with [`#[pyclass(unsendable)]`](../class.md#customizing-the-class), at the cost of making concurrent access to the Rust data be runtime errors. This is only for very specific use cases; it is almost always better to make proper thread-safe types.

## Making `#[pyclass]` types thread-safe

The general challenge with thread-safety is to make sure that two threads cannot produce a data race, i.e. unsynchronized writes to the same data at the same time. A data race produces an unpredictable result and is forbidden by Rust.

By default, `#[pyclass]` employs an ["interior mutability" pattern](../class.md#bound-and-interior-mutability) to allow for either multiple `&T` references or a single exclusive `&mut T` reference to access the data. This allows for simple `#[pyclass]` types to be thread-safe automatically, at the cost of runtime checking for concurrent access. Errors will be raised if the usage overlaps.

For example, the below simple class is thread-safe:

```rust,no_run
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

    fn set_y(&mut self, value: i32) {
        self.y = value;
    }
}
```

In the above example, if calls to `get_x` and `set_y` overlap (from two different threads) then at least one of those threads will experience a runtime error indicating that the data was "already borrowed".

To avoid these errors, you can take control of the interior mutability yourself in one of the following ways.

### Using atomic data structures

To remove the possibility of having overlapping `&self` and `&mut self` references produce runtime errors, consider using `#[pyclass(frozen)]` and use [atomic data structures](https://doc.rust-lang.org/std/sync/atomic/) to control modifications directly.

For example, a thread-safe version of the above `MyClass` using atomic integers would be as follows:

```rust,no_run
# use pyo3::prelude::*;
use std::sync::atomic::{AtomicI32, Ordering};

#[pyclass(frozen)]
struct MyClass {
    x: AtomicI32,
    y: AtomicI32,
}

#[pymethods]
impl MyClass {
    fn get_x(&self) -> i32 {
        self.x.load(Ordering::Relaxed)
    }

    fn set_y(&self, value: i32) {
        self.y.store(value, Ordering::Relaxed)
    }
}
```

### Using locks

An alternative to atomic data structures is to use [locks](https://doc.rust-lang.org/std/sync/struct.Mutex.html) to make threads wait for access to shared data.

For example, a thread-safe version of the above `MyClass` using locks would be as follows:

```rust,no_run
# use pyo3::prelude::*;
use std::sync::Mutex;

struct MyClassInner {
    x: i32,
    y: i32,
}

#[pyclass(frozen)]
struct MyClass {
    inner: Mutex<MyClassInner>
}

#[pymethods]
impl MyClass {
    fn get_x(&self) -> i32 {
        self.inner.lock().expect("lock not poisoned").x
    }

    fn set_y(&self, value: i32) {
        self.inner.lock().expect("lock not poisoned").y = value;
    }
}
```

If you need to lock around state stored in the Python interpreter or otherwise call into the Python C API while a lock is held, you might find the `MutexExt` trait useful. It provides a `lock_py_attached` method for `std::sync::Mutex` that avoids deadlocks with the GIL or other global synchronization events in the interpreter. Additionally, support for the `parking_lot` and `lock_api` synchronization libraries is gated behind the `parking_lot` and `lock_api` features. You can also enable the `arc_lock` feature if you need the `arc_lock` features of either library.

### Wrapping unsynchronized data

In some cases, the data structures stored within a `#[pyclass]` may themselves not be thread-safe. Rust will therefore not implement `Send` and `Sync` on the `#[pyclass]` type.

To achieve thread-safety, a manual `Send` and `Sync` implementation is required which is `unsafe` and should only be done following careful review of the soundness of the implementation. Doing this for PyO3 types is no different than for any other Rust code, [the Rustonomicon](https://doc.rust-lang.org/nomicon/send-and-sync.html) has a great discussion on this.
