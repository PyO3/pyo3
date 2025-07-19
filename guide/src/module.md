# Python modules

You can create a module using `#[pymodule]`:

```rust,no_run
use pyo3::prelude::*;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

/// This module is implemented in Rust.
#[pymodule]
fn my_extension(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)
}
```

The `#[pymodule]` procedural macro takes care of exporting the initialization function of your
module to Python.

The module's name defaults to the name of the Rust function. You can override the module name by
using `#[pyo3(name = "custom_name")]`:

```rust,no_run
use pyo3::prelude::*;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule(name = "custom_name")]
fn my_extension(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(double, m)?)
}
```

The name of the module must match the name of the `.so` or `.pyd`
file. Otherwise, you will get an import error in Python with the following message:
`ImportError: dynamic module does not define module export function (PyInit_name_of_your_module)`

To import the module, either:
 - copy the shared library as described in [Manual builds](building-and-distribution.md#manual-builds), or
 - use a tool, e.g. `maturin develop` with [maturin](https://github.com/PyO3/maturin) or
`python setup.py develop` with [setuptools-rust](https://github.com/PyO3/setuptools-rust).

## Documentation

The [Rust doc comments](https://doc.rust-lang.org/stable/book/ch03-04-comments.html) of the module
initialization function will be applied automatically as the Python docstring of your module.

For example, building off of the above code, this will print `This module is implemented in Rust.`:

```python
import my_extension

print(my_extension.__doc__)
```

## Python submodules

You can create a module hierarchy within a single extension module by using
[`Bound<'_, PyModule>::add_submodule()`]({{#PYO3_DOCS_URL}}/pyo3/prelude/trait.PyModuleMethods.html#tymethod.add_submodule).
For example, you could define the modules `parent_module` and `parent_module.child_module`.

```rust
use pyo3::prelude::*;

#[pymodule]
fn parent_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_child_module(m)?;
    Ok(())
}

fn register_child_module(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let child_module = PyModule::new(parent_module.py(), "child_module")?;
    child_module.add_function(wrap_pyfunction!(func, &child_module)?)?;
    parent_module.add_submodule(&child_module)
}

#[pyfunction]
fn func() -> String {
    "func".to_string()
}

# Python::attach(|py| {
#    use pyo3::wrap_pymodule;
#    use pyo3::types::IntoPyDict;
#    use pyo3::ffi::c_str;
#    let parent_module = wrap_pymodule!(parent_module)(py);
#    let ctx = [("parent_module", parent_module)].into_py_dict(py).unwrap();
#
#    py.run(c_str!("assert parent_module.child_module.func() == 'func'"), None, Some(&ctx)).unwrap();
# })
```

Note that this does not define a package, so this won’t allow Python code to directly import
submodules by using `from parent_module import child_module`. For more information, see
[#759](https://github.com/PyO3/pyo3/issues/759) and
[#1517](https://github.com/PyO3/pyo3/issues/1517#issuecomment-808664021).

It is not necessary to add `#[pymodule]` on nested modules, which is only required on the top-level module.

## Declarative modules

Another syntax based on Rust inline modules is also available to declare modules.

For example:
```rust,no_run
# mod declarative_module_test {
use pyo3::prelude::*;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule]
mod my_extension {
    use super::*;

    #[pymodule_export]
    use super::double; // Exports the double function as part of the module

    #[pymodule_export]
    const PI: f64 = std::f64::consts::PI; // Exports PI constant as part of the module

    #[pyfunction] // This will be part of the module
    fn triple(x: usize) -> usize {
        x * 3
    }

    #[pyclass] // This will be part of the module
    struct Unit;

    #[pymodule]
    mod submodule {
        // This is a submodule
    }

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        // Arbitrary code to run at the module initialization
        m.add("double2", m.getattr("double")?)
    }
}
# }
```

The `#[pymodule]` macro automatically sets the `module` attribute of the `#[pyclass]` macros declared inside of it with its name.
For nested modules, the name of the parent module is automatically added.
In the following example, the `Unit` class will have for `module` `my_extension.submodule` because it is properly nested
but the `Ext` class will have for `module` the default `builtins` because it not nested.

```rust,no_run
# mod declarative_module_module_attr_test {
use pyo3::prelude::*;

#[pyclass]
struct Ext;

#[pymodule]
mod my_extension {
    use super::*;

    #[pymodule_export]
    use super::Ext;

    #[pymodule]
    mod submodule {
        use super::*;
        // This is a submodule

        #[pyclass] // This will be part of the module
        struct Unit;
    }
}
# }
```
It is possible to customize the `module` value for a `#[pymodule]` with the `#[pyo3(module = "MY_MODULE")]` option.

You can provide the `submodule` argument to `pymodule()` for modules that are not top-level modules -- it is automatically set for modules nested inside of a `#[pymodule]`.
