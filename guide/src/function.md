# Python Function

Pyo3 supports two ways to define a function in python. Both require registering
the function to a [module](./module.md)

One way is defining the function in the module definition.

```rust
#![feature(proc_macro)]

extern crate pyo3;
use pyo3::prelude::*;


#[pymodule]
fn rust2py(py: Python, m: &PyModule) -> PyResult<()> {

    // Note that the `#[pyfn()]` annotation automatically converts the arguments from
    // Python objects to Rust values; and the Rust return value back into a Python object.
    #[pyfn(m, "sum_as_string")]
    fn sum_as_string_py(_py: Python, a:i64, b:i64) -> PyResult<String> {
       Ok(format!("{}", a + b).to_string())
    }

    Ok(())
}

# fn main() {}
```

The other is annotating a function with `#[py::function]` and then adding it
to the module using the `add_wrapped_to_module!` macro, which takes the module
as first parameter, the function name as second and an instance of `Python`
as third.

```rust
#![feature(specialization)]

#[macro_use]
extern crate pyo3;
use pyo3::prelude::*;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule]
fn module_with_functions(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_wrapped(wrap_function!(double)).unwrap();

    Ok(())
}

# fn main() {}
```

## Closures

Currently, there are no conversions between `Fn`s in rust and callables in python. This would definitely be possible and very useful, so contributions are welcome. In the meantime, you can do the following:

### Calling a python function in rust

You can use `ObjectProtocol::is_callable` to check if you got a callable, which is true for functions (including lambdas), methods and objects with a `__call__` method. You can call the object with `ObjectProtocol::call` with the args as first parameter and the kwargs (or `NoArgs`) as second paramter. There are also `ObjectProtocol::call0` with no args and `ObjectProtocol::call1` with only the args.

### Calling rust `Fn`s in python

If you have a static function, you can expose it with `#[pyfunction]` and use `wrap_function!` to get the corresponding `PyObject`. For dynamic functions, e.g. lambda and functions that were passed as arguments, you must put them in some kind of owned container, e.g. a box. (Long-Term a special container similar to wasm-bindgen's `Closure` should take care of that). You can than use a `#[pyclass]` struct with that container as field as a way to pass the function over the ffi-barrier. You can even make that class callable with `__call__` so it looks like a function in python code.

