# Python Modules

As shown in the Getting Started chapter, you can create a module as follows:

```rust
use pyo3::prelude::*;

// add bindings to the generated python module
// N.B: names: "librust2py" must be the name of the `.so` or `.pyd` file
/// This module is implemented in Rust.
#[pymodule]
fn rust2py(py: Python, m: &PyModule) -> PyResult<()> {

    // PyO3 aware function. All of our python interfaces could be declared in a separate module.
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

The `#[pymodule]` procedural macro attribute takes care of exporting the initialization function of your module to Python. It can take as an argument the name of your module, which must be the name of the `.so` or `.pyd` file; the default is the Rust function's name.

To import the module, either copy the shared library as described in [Get Started](./overview.md) or use a tool, e.g. `pyo3-pack develop` with [pyo3-pack](https://github.com/PyO3/pyo3-pack) or `python setup.py develop` with [setuptools-rust](https://github.com/PyO3/setuptools-rust).

## Documentation

The [Rust doc comments](https://doc.rust-lang.org/stable/book/first-edition/comments.html) of the module initialization function will be applied automatically as the Python doc string of your module.

```python
import rust2py

print(rust2py.__doc__)
```

Which means that the above Python code will print `This module is implemented in Rust.`.

## Modules as objects

In Python, modules are first class objects. This means that you can store them as values or add them to dicts or other modules:

```rust
use pyo3::prelude::*;
use pyo3::{wrap_pyfunction, wrap_pymodule};
use pyo3::types::IntoPyDict;

#[pyfunction]
fn subfunction() -> String {
    "Subfunction".to_string()
}

#[pymodule]
fn submodule(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pyfunction!(subfunction))?;
    Ok(())
}

#[pymodule]
fn supermodule(_py: Python, module: &PyModule) -> PyResult<()> {
    module.add_wrapped(wrap_pymodule!(submodule))?;
    Ok(())
}

fn nested_call() {
    let gil = GILGuard::acquire();
    let py = gil.python();
    let supermodule = wrap_pymodule!(supermodule)(py);
    let ctx = [("supermodule", supermodule)].into_py_dict(py);

    py.run("assert supermodule.submodule.subfunction() == 'Subfunction'", None, Some(&ctx)).unwrap();
}
```

This way, you can create a module hierarchy within a single extension module.
