# Python Functions

PyO3 supports two ways to define a free function in Python. Both require registering
the function to a [module](./module.md).

One way is defining the function in the module definition, annotated with `#[pyfn]`.

```rust
use pyo3::prelude::*;

#[pymodule]
fn rust2py(py: Python, m: &PyModule) -> PyResult<()> {

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
    m.add_function(wrap_pyfunction!(double, m)?).unwrap();

    Ok(())
}

# fn main() {}
```

## Argument parsing

Both the `#[pyfunction]` and `#[pyfn]` attributes support specifying details of
argument parsing.  The details are given in the section "Method arguments" in
the [Classes](class.md) chapter.  Here is an example for a function that accepts
arbitrary keyword arguments (`**kwargs` in Python syntax) and returns the number
that was passed:

```rust
# extern crate pyo3;
use pyo3::prelude::*;
use pyo3::wrap_pyfunction;
use pyo3::types::PyDict;

#[pyfunction(kwds="**")]
fn num_kwds(kwds: Option<&PyDict>) -> usize {
    kwds.map_or(0, |dict| dict.len())
}

#[pymodule]
fn module_with_functions(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(num_kwds, m)?).unwrap();
    Ok(())
}

# fn main() {}
```

## Making the function signature available to Python

In order to make the function signature available to Python to be retrieved via
`inspect.signature`, use the `#[text_signature]` annotation as in the example
below. The `/` signifies the end of positional-only arguments. (This
is not a feature of this library in particular, but the general format used by
CPython for annotating signatures of built-in functions.)

```rust
use pyo3::prelude::*;

/// This function adds two unsigned 64-bit integers.
#[pyfunction]
#[text_signature = "(a, b, /)"]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
```

This also works for classes and methods:

```rust
use pyo3::prelude::*;
use pyo3::types::PyType;

// it works even if the item is not documented:

#[pyclass]
#[text_signature = "(c, d, /)"]
struct MyClass {}

#[pymethods]
impl MyClass {
    // the signature for the constructor is attached
    // to the struct definition instead.
    #[new]
    fn new(c: i32, d: &str) -> Self {
        Self {}
    }
    // the self argument should be written $self
    #[text_signature = "($self, e, f)"]
    fn my_method(&self, e: i32, f: i32) -> i32 {
        e + f
    }
    #[classmethod]
    #[text_signature = "(cls, e, f)"]
    fn my_class_method(cls: &PyType, e: i32, f: i32) -> i32 {
        e + f
    }
    #[staticmethod]
    #[text_signature = "(e, f)"]
    fn my_static_method(e: i32, f: i32) -> i32 {
        e + f
    }
}
```

Note that `text_signature` on classes is not compatible with compilation in
`abi3` mode until Python 3.10 or greater.

### Making the function signature available to Python (old method)

Alternatively, simply make sure the first line of your docstring is
formatted like in the following example. Please note that the newline after the
`--` is mandatory. The `/` signifies the end of positional-only arguments.

`#[text_signature]` should be preferred, since it will override automatically
generated signatures when those are added in a future version of PyO3.

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

// a function with a signature but without docs. Both blank lines after the `--` are mandatory.

/// sub(a, b, /)
/// --
///
///
#[pyfunction]
fn sub(a: u64, b: u64) -> u64 {
    a - b
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

Currently, there are no conversions between `Fn`s in Rust and callables in Python. This would
definitely be possible and very useful, so contributions are welcome. In the meantime, you can do
the following:

### Calling Python functions in Rust

You can pass Python `def`'d functions and built-in functions to Rust functions [`PyFunction`]
corresponds to regular Python functions while [`PyCFunction`] describes built-ins such as
`repr()`.

You can also use [`PyAny::is_callable`] to check if you have a callable object. `is_callable` will
return `true` for functions (including lambdas), methods and objects with a `__call__` method.
You can call the object with [`PyAny::call`] with the args as first parameter and the kwargs
(or `None`) as second parameter. There are also [`PyAny::call0`] with no args and [`PyAny::call1`]
with only positional args.

### Calling Rust functions in Python

If you have a static function, you can expose it with `#[pyfunction]` and use [`wrap_pyfunction!`]
to get the corresponding [`PyCFunction`]. For dynamic functions, e.g. lambdas and functions that
were passed as arguments, you must put them in some kind of owned container, e.g. a `Box`.
(A long-term solution will be a special container similar to wasm-bindgen's `Closure`). You can
then use a `#[pyclass]` struct with that container as a field as a way to pass the function over
the FFI barrier. You can even make that class callable with `__call__` so it looks like a function
in Python code.

[`PyAny::is_callable`]: https://docs.rs/pyo3/latest/pyo3/struct.PyAny.html#tymethod.is_callable
[`PyAny::call`]: https://docs.rs/pyo3/latest/pyo3/struct.PyAny.html#tymethod.call
[`PyAny::call0`]: https://docs.rs/pyo3/latest/pyo3/struct.PyAny.html#tymethod.call0
[`PyAny::call1`]: https://docs.rs/pyo3/latest/pyo3/struct.PyAny.html#tymethod.call1
[`PyObject`]: https://docs.rs/pyo3/latest/pyo3/type.PyObject.html
[`wrap_pyfunction!`]: https://docs.rs/pyo3/latest/pyo3/macro.wrap_pyfunction.html
[`PyFunction`]: https://docs.rs/pyo3/latest/pyo3/types/struct.PyFunction.html
[`PyCFunction`]: https://docs.rs/pyo3/latest/pyo3/types/struct.PyCFunction.html

### Accessing the module of a function

It is possible to access the module of a `#[pyfunction]` and `#[pyfn]` in the
function body by passing the `pass_module` argument to the attribute:

```rust
use pyo3::wrap_pyfunction;
use pyo3::prelude::*;

#[pyfunction(pass_module)]
fn pyfunction_with_module(module: &PyModule) -> PyResult<&str> {
    module.name()
}

#[pymodule]
fn module_with_fn(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(pyfunction_with_module, m)?)
}

# fn main() {}
```

If `pass_module` is set, the first argument **must** be the `&PyModule`. It is then possible to use the module
in the function body.

The same works for `#[pyfn]`:

```rust
use pyo3::wrap_pyfunction;
use pyo3::prelude::*;

#[pymodule]
fn module_with_fn(py: Python, m: &PyModule) -> PyResult<()> {

    #[pyfn(m, "module_name", pass_module)]
    fn module_name(module: &PyModule) -> PyResult<&str> {
        module.name()
    }
    Ok(())
}

# fn main() {}
```

## Accessing the FFI functions

In order to make Rust functions callable from Python, PyO3 generates a
`extern "C" Fn(slf: *mut PyObject, args: *mut PyObject, kwargs: *mut PyObject) -> *mut Pyobject`
function and embeds the call to the Rust function inside this FFI-wrapper function. This
wrapper handles extraction of the regular arguments and the keyword arguments from the input
`PyObjects`. Since this function is not user-defined but required to build a `PyCFunction`, PyO3
offers the `raw_pycfunction!()` macro to get the identifier of this generated wrapper.

The `wrap_pyfunction` macro can be used to directly get a `PyCFunction` given a
`#[pyfunction]` and a `PyModule`: `wrap_pyfunction!(rust_fun, module)`.
