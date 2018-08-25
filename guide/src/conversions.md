# Type Conversions

`PyO3` provides some handy traits to convert between Python types and Rust types.

## `.extract()?`

The easiest way to convert a python object to a rust value is using `.extract()`.

## `ToPyObject` and `IntoPyObject` trait

[`ToPyObject`] trait is a conversion trait that allows various objects to be converted into [`PyObject`][PyObject]. [`IntoPyObject`][IntoPyObject] serves the same purpose except it consumes `self`.

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

## `*args` and `**kwargs` for python object call

There are several way how to pass positional and keyword arguments to python object call.
[`ObjectProtocol`][ObjectProtocol] trait
provides two methods:

* `call` - call callable python object.
* `call_method` - call specific method on the object.

Both methods accept `args` and `kwargs` arguments. `args` argument is generate over
[`IntoPyTuple`][IntoPyTuple] trait. So args could be `PyTuple` instance or
rust tuple with up to 10 elements. Or `NoArgs` object which represents empty tuple object.

```rust
extern crate pyo3;
use pyo3::prelude::*;

# struct SomeObject;
# impl SomeObject {
#     fn new(py: Python) -> PyObject {
#           pyo3::PyDict::new(py).to_object(py)
#     }
# }
#
fn main() {
    # let arg1 = "arg1";
    # let arg2 = "arg2";
    # let arg3 = "arg3";

    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = SomeObject::new(py);

    // call object without empty arguments
    obj.call0(py);

    // call object with PyTuple
    let args = PyTuple::new(py, &[arg1, arg2, arg3]);
    obj.call1(py, args);

    // pass arguments as rust tuple
    let args = (arg1, arg2, arg3);
    obj.call1(py, args);
}
```

`kwargs` can by `None` or `Some(PyDict)`. You can use the 
[`IntoPyDict`][IntoPyDict] trait to convert other dict-like containers, e.g. `HashMap`, `BTreeMap` as well as tuples with up to 10 elements and `Vec`s where each element is a two element tuple.

```rust
extern crate pyo3;

use std::collections::HashMap;
use pyo3::prelude::*;

# struct SomeObject;
# impl SomeObject {
#     fn new(py: Python) -> PyObject {
#           pyo3::PyDict::new(py).to_object(py)
#     }
# }
fn main() {
    # let key1 = "key1";
    # let val1 = 1;
    # let key2 = "key2";
    # let val2 = 2;

    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = SomeObject::new(py);

    // call object with PyDict
    let kwargs = PyDict::new(py);
    kwargs.set_item(key1, val1);
    obj.call(py, NoArgs, kwargs);

    // pass arguments as rust tuple
    let kwargs = ((key1, val1), (key2, val2));
    obj.call(py, NoArgs, kwargs);

    // pass arguments as HashMap
    let mut kwargs = HashMap::<&str, i32>::new();
    kwargs.insert(key1, 1);
    obj.call(py, NoArgs, kwargs);
}
```


TODO

[`ToPyObject`]: https://docs.rs/pyo3/0.2.7/trait.ToPyObject.html
[IntoPyObject]: https://docs.rs/pyo3/0.2.7/trait.IntoPyObject.html
[PyObject]: https://docs.rs/pyo3/0.2.7/struct.PyObject.html
[IntoPyTuple]: https://docs.rs/pyo3/0.2.7/trait.IntoPyTuple.html
[PyTuple]: https://docs.rs/pyo3/0.2.7/struct.PyTuple.html
[ObjectProtocol]: https://docs.rs/pyo3/0.2.7/trait.ObjectProtocol.html
[IntoPyDict]: https://docs.rs/pyo3/0.2.7/trait.IntoPyDict.html
