# Type Conversions

PyO3 provides some handy traits to convert between Python types and Rust types.

## `.extract()` and the `FromPyObject` trait

The easiest way to convert a Python object to a Rust value is using
`.extract()`.  It returns a `PyResult` with a type error if the conversion
fails, so usually you will use something like

```ignore
let v: Vec<i32> = obj.extract()?;
```

This method is available for many Python object types, and can produce a wide
variety of Rust types, which you can check out in the implementor list of
[`FromPyObject`].

[`FromPyObject`] is also implemented for your own Rust types wrapped as Python
objects (see [the chapter about classes](class.md)).  There, in order to both be
able to operate on mutable references *and* satisfy Rust's rules of non-aliasing
mutable references, you have to extract the PyO3 reference wrappers [`PyRef`]
and [`PyRefMut`].  They work like the reference wrappers of
`std::cell::RefCell` and ensure (at runtime) that Rust borrows are allowed.


## The `ToPyObject` trait

[`ToPyObject`] is a conversion trait that allows various objects to be
converted into [`PyObject`]. `IntoPy<PyObject>` serves the
same purpose, except that it consumes `self`.


## `*args` and `**kwargs` for Python object calls

There are several ways how to pass positional and keyword arguments to a Python object call.
The [`ObjectProtocol`] trait provides two methods:

* `call` - call any callable Python object.
* `call_method` - call a specific method on the object, shorthand for `get_attr` then `call`.

Both methods need `args` and `kwargs` arguments, but there are variants for less
complex calls, such as `call1` for only `args` and `call0` for no arguments at all.

```rust
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};

struct SomeObject;
impl SomeObject {
    fn new(py: Python) -> PyObject {
        PyDict::new(py).to_object(py)
    }
}

fn main() {
    let arg1 = "arg1";
    let arg2 = "arg2";
    let arg3 = "arg3";

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

`kwargs` can be `None` or `Some(&PyDict)`. You can use the
[`IntoPyDict`] trait to convert other dict-like containers,
e.g. `HashMap` or `BTreeMap`, as well as tuples with up to 10 elements and
`Vec`s where each element is a two-element tuple.

```rust
use pyo3::prelude::*;
use pyo3::types::{IntoPyDict, PyDict};
use std::collections::HashMap;

struct SomeObject;

impl SomeObject {
    fn new(py: Python) -> PyObject {
        PyDict::new(py).to_object(py)
    }
}

fn main() {
    let key1 = "key1";
    let val1 = 1;
    let key2 = "key2";
    let val2 = 2;

    let gil = Python::acquire_gil();
    let py = gil.python();

    let obj = SomeObject::new(py);

    // call object with PyDict
    let kwargs = [(key1, val1)].into_py_dict(py);
    obj.call(py, (), Some(kwargs));

    // pass arguments as Vec
    let kwargs = vec![(key1, val1), (key2, val2)];
    obj.call(py, (), Some(kwargs.into_py_dict(py)));

    // pass arguments as HashMap
    let mut kwargs = HashMap::<&str, i32>::new();
    kwargs.insert(key1, 1);
    obj.call(py, (), Some(kwargs.into_py_dict(py)));
}
```

## `FromPy<T>` and `IntoPy<T>`

Many conversions in PyO3 can't use `std::convert::From` because they need a GIL token.
The [`FromPy`] trait offers an `from_py` method that works just like `from`, except for taking a `Python<'_>` argument.
I.e. `FromPy<T>` could be converting a Rust object into a Python object even though it is called [`FromPy`] - it doesn't say anything about which side of the conversion is a Python object.

Just like `From<T>`, if you implement `FromPy<T>` you gain a blanket implementation of [`IntoPy`] for free.

Eventually, traits such as [`ToPyObject`] will be replaced by this trait and a [`FromPy`] trait will be added that will implement
[`IntoPy`], just like with `From` and `Into`.

[`IntoPy`]: https://docs.rs/pyo3/latest/pyo3/trait.IntoPy.html
[`FromPy`]: https://docs.rs/pyo3/latest/pyo3/trait.FromPy.html
[`FromPyObject`]: https://docs.rs/pyo3/latest/pyo3/types/trait.FromPyObject.html
[`ToPyObject`]: https://docs.rs/pyo3/latest/pyo3/trait.ToPyObject.html
[`PyObject`]: https://docs.rs/pyo3/latest/pyo3/struct.PyObject.html
[`PyTuple`]: https://docs.rs/pyo3/latest/pyo3/types/struct.PyTuple.html
[`ObjectProtocol`]: https://docs.rs/pyo3/latest/pyo3/trait.ObjectProtocol.html
[`IntoPyDict`]: https://docs.rs/pyo3/latest/pyo3/types/trait.IntoPyDict.html

[`PyRef`]: https://pyo3.rs/master/doc/pyo3/pycell/struct.PyRef.html
[`PyRefMut`]: https://pyo3.rs/master/doc/pyo3/pycell/struct.PyRefMut.html
