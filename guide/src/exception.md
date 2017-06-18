# Python Exception

## Define a new exception

You can use the `py_exception!` macro to define a new excetpion type:

```rust
py_exception!(module, MyError);
```

* `module` is the name of the containing module.
* `MyError` is the name of the new exception type.

For example:

```rust
#[macro_use] extern crate pyo3;

use pyo3::{Python, PyDict};

py_exception!(mymodule, CustomError);

fn main() {
let gil = Python::acquire_gil();
    let py = gil.python();
    let ctx = PyDict::new(py);

    ctx.set_item(py, "CustomError", py.get_type::<CustomError>()).unwrap();

    py.run("assert str(CustomError) == \"<class 'mymodule.CustomError'>\"", None, Some(&ctx)).unwrap();
    py.run("assert CustomError('oops').args == ('oops',)", None, Some(&ctx)).unwrap();
}
```

## Raise an exception

TODO

## Check exception type

TODO
