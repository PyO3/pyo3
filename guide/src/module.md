# Python Module

As shown in the Getting Started chapter, you can create a module as follows:

```rust
#![feature(proc_macro)]

extern crate pyo3;
use pyo3::{PyResult, Python, PyModule};

use pyo3::pymodinit;

// add bindings to the generated python module
// N.B: names: "librust2py" must be the name of the `.so` or `.pyd` file
/// This module is implemented in Rust.
#[pymodinit(rust2py)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {

    // pyo3 aware function. All of our python interface could be declared in a separate module.
    // Note that the `#[pyfn()]` annotation automatically converts the arguments from
    // Python objects to Rust values; and the Rust return value back into a Python object.
    #[pyfn(m, "sum_as_string")]
    fn sum_as_string_py(_py: Python, a:i64, b:i64) -> PyResult<String> {
       let out = sum_as_string(a, b);
       Ok(out)
    }

    Ok(())
}

// logic implemented as a normal rust function
fn sum_as_string(a:i64, b:i64) -> String {
    format!("{}", a + b).to_string()
}

# fn main() {}
```

The `#[pymodinit}` procedural macro attribute takes care of exporting the initialization function of your module to Python. It takes one argument as the name of your module, it must be the name of the `.so` or `.pyd` file.

The [Rust doc comments](https://doc.rust-lang.org/stable/book/first-edition/comments.html) of the module initialization function will be applied automatically as the Python doc string of your module.

```python
import rust2py

print(rust2py.__doc__)
```

Which means that the above Python code will print `This module is implemented in Rust.`.

> On macOS, you will need to rename the output from `*.dylib` to `*.so`.
>
> On Windows, you will need to rename the output from `*.dll` to `*.pyd`.

For `setup.py` integration, You can use [setuptools-rust](https://github.com/PyO3/setuptools-rust),
learn more about it in [Distribution](./distribution.html).
