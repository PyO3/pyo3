# Calling Python functions

The `Bound<'py, T>` smart pointer (such as `Bound<'py, PyAny>`, `Bound<'py, PyList>`, or `Bound<'py, MyClass>`) can be used to call Python functions.

PyO3 offers two APIs to make function calls:

* [`call`]({{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.call) - call any callable Python object.
* [`call_method`]({{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.call_method) - call a method on the Python object.

Both of these APIs take `args` and `kwargs` arguments (for positional and keyword arguments respectively). There are variants for less complex calls:

* [`call1`]({{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.call1) and [`call_method1`]({{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.call_method1) to call only with positional `args`.
* [`call0`]({{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.call0) and [`call_method0`]({{#PYO3_DOCS_URL}}/pyo3/types/trait.PyAnyMethods.html#tymethod.call_method0) to call with no arguments.

For convenience the [`Py<T>`](../types.md#pyt) smart pointer also exposes these same six API methods, but needs a `Python` token as an additional first argument to prove the GIL is held.

The example below calls a Python function behind a `Py<PyAny>` reference:

```rust
use pyo3::prelude::*;
use pyo3::types::PyTuple;
use pyo3_ffi::c_str;

fn main() -> PyResult<()> {
    let arg1 = "arg1";
    let arg2 = "arg2";
    let arg3 = "arg3";

    Python::attach(|py| {
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            c_str!("def example(*args, **kwargs):
                if args != ():
                    print('called with args', args)
                if kwargs != {}:
                    print('called with kwargs', kwargs)
                if args == () and kwargs == {}:
                    print('called with no arguments')"),
            c_str!("example.py"),
            c_str!(""),
        )?
        .getattr("example")?
        .into();

        // call object without any arguments
        fun.call0(py)?;

        // pass object with Rust tuple of positional arguments
        let args = (arg1, arg2, arg3);
        fun.call1(py, args)?;

        // call object with Python tuple of positional arguments
        let args = PyTuple::new(py, &[arg1, arg2, arg3])?;
        fun.call1(py, args)?;
        Ok(())
    })
}
```

## Creating keyword arguments

For the `call` and `call_method` APIs, `kwargs` are `Option<&Bound<'py, PyDict>>`, so can either be `None` or `Some(&dict)`. You can use the [`IntoPyDict`]({{#PYO3_DOCS_URL}}/pyo3/types/trait.IntoPyDict.html) trait to convert other dict-like containers, e.g. `HashMap` or `BTreeMap`, as well as tuples with up to 10 elements and `Vec`s where each element is a two-element tuple. To pass keyword arguments of different types, construct a `PyDict` object.

```rust
use pyo3::prelude::*;
use pyo3::types::{PyDict, IntoPyDict};
use std::collections::HashMap;
use pyo3::ffi::c_str;

fn main() -> PyResult<()> {
    let key1 = "key1";
    let val1 = 1;
    let key2 = "key2";
    let val2 = 2;

    Python::attach(|py| {
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            c_str!("def example(*args, **kwargs):
                if args != ():
                    print('called with args', args)
                if kwargs != {}:
                    print('called with kwargs', kwargs)
                if args == () and kwargs == {}:
                    print('called with no arguments')"),
            c_str!("example.py"),
            c_str!(""),
        )?
        .getattr("example")?
        .into();

        // call object with PyDict
        let kwargs = [(key1, val1)].into_py_dict(py)?;
        fun.call(py, (), Some(&kwargs))?;

        // pass arguments as Vec
        let kwargs = vec![(key1, val1), (key2, val2)];
        fun.call(py, (), Some(&kwargs.into_py_dict(py)?))?;

        // pass arguments as HashMap
        let mut kwargs = HashMap::<&str, i32>::new();
        kwargs.insert(key1, 1);
        fun.call(py, (), Some(&kwargs.into_py_dict(py)?))?;

        // pass arguments of different types as PyDict
        let kwargs = PyDict::new(py);
        kwargs.set_item(key1, val1)?;
        kwargs.set_item(key2, "string")?;
        fun.call(py, (), Some(&kwargs))?;

        Ok(())
    })
}
```
