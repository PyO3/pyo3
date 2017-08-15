# Python Module

Python module generation is powered by unstable [Procedural Macros](https://doc.rust-lang.org/book/first-edition/procedural-macros.html) feature, so you need to turn on `proc_macro` feature:

```rust
#![feature(proc_macro)]

extern crate pyo3;
# fn main() {}
```

You need to change your `crate-type` to `cdylib` to produce a Python compatible library:

```toml
[lib]
name = "rust2py"
crate-type = ["cdylib"]

[dependencies]
pyo3 = { version = "0.2", features = ["extension-module"] }
```

Now you can write your module, for example

```rust
#![feature(proc_macro)]

extern crate pyo3;
use pyo3::{py, PyResult, Python, PyModule};

// add bindings to the generated python module
// N.B: names: "librust2py" must be the name of the `.so` or `.pyd` file
/// This module is implemented in Rust.
#[py::modinit(rust2py)]
fn init_mod(py: Python, m: &PyModule) -> PyResult<()> {

    // pyo3 aware function. All of our python interface could be declared in a separate module.
    // Note that the `#[pyfn()]` annotation automatically converts the arguments from
    // Python objects to Rust values; and the Rust return value back into a Python object.
    #[pyfn(m, "sum_as_string")]
    fn sum_as_string_py(_: Python, a:i64, b:i64) -> PyResult<String> {
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

The `modinit` procedural macro attribute takes care of exporting the initialization function of your module to Python. It takes one argument as the name of your module, it must be the name of the `.so` or `.pyd` file.

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
