# Python modules

You can create a module using `#[pymodule]`:

```rust,no_run
# mod declarative_module_basic_test {
use pyo3::prelude::*;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

/// This module is implemented in Rust.
#[pymodule]
mod my_extension {
    use pyo3::prelude::*;

    #[pymodule_export]
    use super::double; // The double function is made available from Python, works also with classes

    #[pyfunction] // Inline definition of a pyfunction, also made available to Python
    fn triple(x: usize) -> usize {
        x * 3
    }
}
# }
```

The `#[pymodule]` procedural macro takes care of creating the initialization function of your module and exposing it to Python.

The module's name defaults to the name of the Rust module.
You can override the module name by using `#[pyo3(name = "custom_name")]`:

```rust,no_run
# mod declarative_module_custom_name_test {
use pyo3::prelude::*;

#[pyfunction]
fn double(x: usize) -> usize {
    x * 2
}

#[pymodule(name = "custom_name")]
mod my_extension {
    #[pymodule_export]
    use super::double;
}
# }
```

The name of the module must match the name of the `.so` or `.pyd` file.
Otherwise, you will get an import error in Python with the following message: `ImportError: dynamic module does not define module export function (PyInit_name_of_your_module)`

To import the module, either:

- copy the shared library as described in [Manual builds](building-and-distribution.md#manual-builds), or
- use a tool, e.g. `maturin develop` with [maturin](https://github.com/PyO3/maturin) or
`python setup.py develop` with [setuptools-rust](https://github.com/PyO3/setuptools-rust).

## Documentation

The [Rust doc comments](https://doc.rust-lang.org/stable/book/ch03-04-comments.html) of the Rust module will be applied automatically as the Python docstring of your module.

For example, building off of the above code, this will print `This module is implemented in Rust.`:

```python
import my_extension

print(my_extension.__doc__)
```

## Python submodules

You can create a module hierarchy within a single extension module by just `use`ing modules like functions or classes.
For example, you could define the modules `parent_module` and `parent_module.child_module`:

```rust
use pyo3::prelude::*;

#[pymodule]
mod parent_module {
    #[pymodule_export]
    use super::child_module;
}

#[pymodule]
mod child_module {
    #[pymodule_export]
    use super::func;
}

#[pyfunction]
fn func() -> String {
    "func".to_string()
}
#
# fn main() {
#   Python::attach(|py| {
#       use pyo3::wrap_pymodule;
#       use pyo3::types::IntoPyDict;
#       let parent_module = wrap_pymodule!(parent_module)(py);
#       let ctx = [("parent_module", parent_module)].into_py_dict(py).unwrap();
#
#      py.run(c"assert parent_module.child_module.func() == 'func'", None, Some(&ctx)).unwrap();
#   })
# }
```

Note that this does not define a package, so this won’t allow Python code to directly import submodules by using `from parent_module import child_module`.
For more information, see [#759](https://github.com/PyO3/pyo3/issues/759) and [#1517](https://github.com/PyO3/pyo3/issues/1517#issuecomment-808664021).

You can provide the `submodule` argument to `#[pymodule()]` for modules that are not top-level modules in order for them to properly generate the `#[pyclass]` `module` attribute automatically.

## Inline declaration

It is possible to declare functions, classes, sub-modules and constants inline in a module:

For example:

```rust,no_run
# mod declarative_module_test {
#[pyo3::pymodule]
mod my_extension {
    use pyo3::prelude::*;

    #[pymodule_export]
    const PI: f64 = std::f64::consts::PI; // Exports PI constant as part of the module

    #[pyfunction] // This will be part of the module
    fn double(x: usize) -> usize {
        x * 2
    }

    #[pyclass] // This will be part of the module
    struct Unit;

    #[pymodule]
    mod submodule {
        // This is a submodule
        use pyo3::prelude::*;

        #[pyclass]
        struct Nested;
    }
}
# }
```

In this case, `#[pymodule]` macro automatically sets the `module` attribute of the `#[pyclass]` macros declared inside of it with its name.
For nested modules, the name of the parent module is automatically added.
In the previous example, the `Nested` class will have for `module` `my_extension.submodule`.

## Procedural initialization

If the macros provided by PyO3 are not enough, it is possible to run code at the module initialization:

```rust,no_run
# mod procedural_module_test {
#[pyo3::pymodule]
mod my_extension {
    use pyo3::prelude::*;

    #[pyfunction]
    fn double(x: usize) -> usize {
        x * 2
    }

    #[pymodule_init]
    fn init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        // Arbitrary code to run at the module initialization
        m.add("double2", m.getattr("double")?)
    }
}
# }
```
