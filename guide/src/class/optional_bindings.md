# Optional bindings

You might want to write a library the is usable both in pure rust and as a python library. For that, pyo3 support wrapping attributes in `#[cfg_attr(feature = "pyo3", ...)]` (the feature unfortunately has to be hardcoded, so the feature must be named `pyo3`). This does not only apply to classes and their methods but also to e.g. `#[pyfunction]`.

Make pyo3 optional in Cargo.toml:

```toml
[dependencies]
pyo3 = { version = "0.14", features = ["extension-module", "abi3"], optional = true }
```

If you're using maturin, also set `pyo3` as a default feature in pyproject.toml, so `maturin build` will work as well as `cargo build`:

```toml
[tool.maturin]
features = ["pyo3"]
```

Implementing a `Number` again, but this time making all attributes and the module function optional:

```rust
use pyo3::prelude::*;

#[cfg_attr(feature = "pyo3", pyclass)]
struct Number(i32);

#[cfg_attr(feature = "pyo3", pymethods)]
impl Number {
    #[cfg_attr(feature = "pyo3", classattr)]
    const SMALLEST_PRIME: i32 = 2;

    #[cfg_attr(feature = "pyo3", new)]
    fn new(value: i32) -> Self {
        Self(value)
    }

    /// Computes the [Greatest common divisor](https://en.wikipedia.org/wiki/Greatest_common_divisor) of two numbers
    #[cfg_attr(feature = "pyo3", pyo3(name = "gcd"))]
    fn greatest_common_divisor(&self, other: &Self) -> Self {
        let mut a = self.0; 
        let mut b = other.0;
        while a != b {
            if a > b {
                a -= b
            } else {
                b -= a
            }
        }

        Self::new(a)
    }
}

#[cfg(feature = "pyo3")] // We don't want that function at all in a rust library
#[pymodule]
fn my_module(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<Number>()?;
    Ok(())
}
```

Now you have a library that you can use both normally in rust without any python dependency and as a python library. 