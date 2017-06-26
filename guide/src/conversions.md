# Type Conversions

`PyO3` provides some handy traits to convert between Python types and Rust types.

## `ToPyObject` and `IntoPyObject` trait

[`ToPyObject`][ToPyObject] trait is a conversion trait that allows various objects to be converted into [`PyObject`][PyObject]. [`IntoPyObject`][IntoPyObject] serves the same purpose except it consumes `self`.

## `IntoPyTuple` trait

[`IntoPyTuple`][IntoPyTuple] trait is a conversion trait that allows various objects to be converted into [`PyTuple`][PyTuple] object.

For example, [`IntoPyTuple`][IntoPyTuple] trait is implemented for `()` so that you can convert it into a empty [`PyTuple`][PyTuple]

```rust
extern crate pyo3;

use pyo3::{Python, IntoPyTuple};

fn main() {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let py_tuple = ().into_tuple(py);
}
```

## `FromPyObject` and `RefFromPyObject` trait

TODO

[ToPyObject]: https://pyo3.github.io/PyO3/pyo3/trait.ToPyObject.html
[IntoPyObject]: https://pyo3.github.io/PyO3/pyo3/trait.IntoPyObject.html
[PyObject]: https://pyo3.github.io/PyO3/pyo3/struct.PyObject.html
[IntoPyTuple]: https://pyo3.github.io/PyO3/pyo3/trait.IntoPyTuple.html
[PyTuple]: https://pyo3.github.io/PyO3/pyo3/struct.PyTuple.html
