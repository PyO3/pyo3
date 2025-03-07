# Emulating callable objects

Classes can be callable if they have a `#[pymethod]` named `__call__`.
This allows instances of a class to behave similar to functions.

This method's signature must look like `__call__(<self>, ...) -> object` - here,
 any argument list can be defined as for normal pymethods

### Example: Implementing a call counter

The following pyclass is a basic decorator - its constructor takes a Python object
as argument and calls that object when called. An equivalent Python implementation
is linked at the end.

An example crate containing this pyclass can be found [here](https://github.com/PyO3/pyo3/tree/main/examples/decorator)

```rust,ignore
{{#include ../../../examples/decorator/src/lib.rs}}
```

Python code:

```python
{{#include ../../../examples/decorator/tests/example.py}}
```

Output:

```text
say_hello has been called 1 time(s).
hello
say_hello has been called 2 time(s).
hello
say_hello has been called 3 time(s).
hello
say_hello has been called 4 time(s).
hello
```

### Pure Python implementation

A Python implementation of this looks similar to the Rust version:

```python
class Counter:
    def __init__(self, wraps):
        self.count = 0
        self.wraps = wraps

    def __call__(self, *args, **kwargs):
        self.count += 1
        print(f"{self.wraps.__name__} has been called {self.count} time(s)")
        self.wraps(*args, **kwargs)
```

Note that it can also be implemented as a higher order function:

```python
def Counter(wraps):
    count = 0
    def call(*args, **kwargs):
        nonlocal count
        count += 1
        print(f"{wraps.__name__} has been called {count} time(s)")
        return wraps(*args, **kwargs)
    return call
```

### What is the `AtomicU64` for?

A [previous implementation] used a normal `u64`, which meant it required a `&mut self` receiver to update the count:

```rust,ignore
#[pyo3(signature = (*args, **kwargs))]
fn __call__(
    &mut self,
    py: Python<'_>,
    args: &Bound<'_, PyTuple>,
    kwargs: Option<&Bound<'_, PyDict>>,
) -> PyResult<Py<PyAny>> {
    self.count += 1;
    let name = self.wraps.getattr(py, "__name__")?;

    println!("{} has been called {} time(s).", name, self.count);

    // After doing something, we finally forward the call to the wrapped function
    let ret = self.wraps.call(py, args, kwargs)?;

    // We could do something with the return value of
    // the function before returning it
    Ok(ret)
}
```

The problem with this is that the `&mut self` receiver means PyO3 has to borrow it exclusively,
 and hold this borrow across the`self.wraps.call(py, args, kwargs)` call. This call returns control to the user's Python code
 which is free to call arbitrary things, *including* the decorated function. If that happens PyO3 is unable to create a second unique borrow and will be forced to raise an exception.

As a result, something innocent like this will raise an exception:

```py
@Counter
def say_hello():
    if say_hello.count < 2:
        print(f"hello from decorator")

say_hello()
# RuntimeError: Already borrowed
```

The implementation in this chapter fixes that by never borrowing exclusively; all the methods take `&self` as receivers, of which multiple may exist simultaneously. This requires a shared counter and the most straightforward way to implement thread-safe interior mutability (e.g. the type does not need to accept `&mut self` to modify the "interior" state) for a `u64` is to use [`AtomicU64`], so that's what is used here.

This shows the dangers of running arbitrary Python code - note that "running arbitrary Python code" can be far more subtle than the example above:
- Python's asynchronous executor may park the current thread in the middle of Python code, even in Python code that *you* control, and let other Python code run.
- Dropping arbitrary Python objects may invoke destructors defined in Python (`__del__` methods).
- Calling Python's C-api (most PyO3 apis call C-api functions internally) may raise exceptions, which may allow Python code in signal handlers to run.
- On the free-threaded build, users might use Python's `threading` module to work with your types simultaneously from multiple OS threads.

This is especially important if you are writing unsafe code; Python code must never be able to cause undefined behavior. You must ensure that your Rust code is in a consistent state before doing any of the above things.

[previous implementation]: https://github.com/PyO3/pyo3/discussions/2598 "Thread Safe Decorator <Help Wanted> · Discussion #2598 · PyO3/pyo3"
[`AtomicU64`]: https://doc.rust-lang.org/std/sync/atomic/struct.AtomicU64.html "AtomicU64 in std::sync::atomic - Rust"
