# Python Functions

PyO3 supports two ways to define a free function in Python. Both require registering
the function to a [module](./module.md).

One way is defining the function in the module definition.

```rust
use pyo3::prelude::*;

#[pymodule]
fn rust2py(py: Python, m: &PyModule) -> PyResult<()> {

    // Note that the `#[pyfn()]` annotation automatically converts the arguments from
    // Python objects to Rust values; and the Rust return value back into a Python object.
    #[pyfn(m, "sum_as_string")]
    fn sum_as_string_py(_py: Python, a:i64, b:i64) -> PyResult<String> {
       Ok(format!("{}", a + b))
    }

    Ok(())
}

# fn main() {}
```

The other is annotating a function with `#[pyfunction]` and then adding it
to the module using the `wrap_pyfunction!` macro.

```rust
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule]
fn module_with_functions(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_pyfunction!(double)).unwrap();

    Ok(())
}

# fn main() {}
```

### Making the function signature available to Python

In order to make the function signature available to Python to be retrieved via
`inspect.signature`, simply make sure the first line of your docstring is
formatted like in the example below. Please note that the newline after the
`--` is mandatory. The `/` signifies the end of positional-only arguments. This
is not a feature of this library in particular, but the general format used by
CPython for annotating signatures of built-in functions. Function signatures for
built-ins are new in Python 3 — in Python 2, they are simply considered to be a
part of the docstring.

```rust
use pyo3::prelude::*;

/// add(a, b, /)
/// --
///
/// This function adds two unsigned 64-bit integers.
#[pyfunction]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
```

When annotated like this, signatures are also correctly displayed in IPython.
```ignore
>>> pyo3_test.add?
Signature: pyo3_test.add(a, b, /)
Docstring: This function adds two unsigned 64-bit integers.
Type:      builtin_function_or_method
```

## Closures

Currently, there are no conversions between `Fn`s in Rust and callables in Python. This would definitely be possible and very useful, so contributions are welcome. In the meantime, you can do the following:

### Calling a Python function in Rust

You can use `ObjectProtocol::is_callable` to check if you got a callable, which is true for functions (including lambdas), methods and objects with a `__call__` method. You can call the object with `ObjectProtocol::call` with the args as first parameter and the kwargs (or `None`) as second parameter. There are also `ObjectProtocol::call0` with no args and `ObjectProtocol::call1` with only the positional args.

### Calling Rust `Fn`s in Python

If you have a static function, you can expose it with `#[pyfunction]` and use `wrap_pyfunction!` to get the corresponding `PyObject`. For dynamic functions, e.g. lambda and functions that were passed as arguments, you must put them in some kind of owned container, e.g. a box. (A long-term solution will be a special container similar to wasm-bindgen's `Closure`). You can then use a `#[pyclass]` struct with that container as a field as a way to pass the function over the FFI barrier. You can even make that class callable with `__call__` so it looks like a function in Python code.
