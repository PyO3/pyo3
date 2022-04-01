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

```rust
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

#### Pure Python implementation

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
