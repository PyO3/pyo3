# Python Function

Pyo3 supports two ways to define a function in python. Both require registering
the function to a [module](./module.md)

One way is defining the function in the module definition.

```rust
#![feature(proc_macro)]

extern crate pyo3;
use pyo3::prelude::*;
use pyo3::pymodinit;

#[pymodinit(rust2py)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {

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
to the module using the `add_function_to_module!` macro, which takes the module
as first parameter, the function name as second and an instance of `Python`
as third.

```rust
#![feature(proc_macro, concat_idents)]

#[macro_use]
extern crate pyo3;
use pyo3::prelude::*;

use pyo3::{pyfunction, pymodinit};

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodinit(module_with_functions)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_function!(double)).unwrap();

    Ok(())
}

# fn main() {}
```

