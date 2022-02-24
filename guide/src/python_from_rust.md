# Calling Python in Rust code

This chapter of the guide documents some ways to interact with Python code from Rust:
 - How to call Python functions
 - How to execute existing Python code

## Calling Python functions

Any Python-native object reference (such as `&PyAny`, `&PyList`, or `&PyCell<MyClass>`) can be used to call Python functions.

PyO3 offers two APIs to make function calls:

* [`call`](https://docs.rs/pyo3/0.12.3/pyo3/struct.PyAny.html#method.call) - call any callable Python object.
* [`call_method`](https://docs.rs/pyo3/0.12.3/pyo3/struct.PyAny.html#method.call_method) - call a method on the Python object.

Both of these APIs take `args` and `kwargs` arguments (for positional and keyword arguments respectively). There are variants for less complex calls:

* [`call1`](https://docs.rs/pyo3/0.12.3/pyo3/struct.PyAny.html#method.call1) and [`call_method1`](https://docs.rs/pyo3/0.12.3/pyo3/struct.PyAny.html#method.call_method1) to call only with positional `args`.
* [`call0`](https://docs.rs/pyo3/0.12.3/pyo3/struct.PyAny.html#method.call0) and [`call_method0`](https://docs.rs/pyo3/0.12.3/pyo3/struct.PyAny.html#method.call_method0) to call with no arguments.

For convenience the [`Py<T>`](types.html#pyt-and-pyobject) smart pointer also exposes these same six API methods, but needs a `Python` token as an additional first argument to prove the GIL is held.

The example below calls a Python function behind a `PyObject` (aka `Py<PyAny>`) reference:

```rust
use pyo3::prelude::*;
use pyo3::types::PyTuple;

fn main() -> PyResult<()> {
    let arg1 = "arg1";
    let arg2 = "arg2";
    let arg3 = "arg3";

    Python::with_gil(|py| {
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            "def example(*args, **kwargs):
                if args != ():
                    print('called with args', args)
                if kwargs != {}:
                    print('called with kwargs', kwargs)
                if args == () and kwargs == {}:
                    print('called with no arguments')",
            "",
            "",
        )?.getattr("example")?.into();

        // call object without empty arguments
        fun.call0(py)?;

        // call object with PyTuple
        let args = PyTuple::new(py, &[arg1, arg2, arg3]);
        fun.call1(py, args)?;

        // pass arguments as rust tuple
        let args = (arg1, arg2, arg3);
        fun.call1(py, args)?;
        Ok(())
    })
}
```

### Creating keyword arguments

For the `call` and `call_method` APIs, `kwargs` can be `None` or `Some(&PyDict)`. You can use the [`IntoPyDict`]({{#PYO3_DOCS_URL}}/pyo3/types/trait.IntoPyDict.html) trait to convert other dict-like containers, e.g. `HashMap` or `BTreeMap`, as well as tuples with up to 10 elements and `Vec`s where each element is a two-element tuple.

```rust
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;
use std::collections::HashMap;

fn main() -> PyResult<()> {
    let key1 = "key1";
    let val1 = 1;
    let key2 = "key2";
    let val2 = 2;

    Python::with_gil(|py| {
        let fun: Py<PyAny> = PyModule::from_code(
            py,
            "def example(*args, **kwargs):
                if args != ():
                    print('called with args', args)
                if kwargs != {}:
                    print('called with kwargs', kwargs)
                if args == () and kwargs == {}:
                    print('called with no arguments')",
            "",
            "",
        )?.getattr("example")?.into();


        // call object with PyDict
        let kwargs = [(key1, val1)].into_py_dict(py);
        fun.call(py, (), Some(kwargs))?;

        // pass arguments as Vec
        let kwargs = vec![(key1, val1), (key2, val2)];
        fun.call(py, (), Some(kwargs.into_py_dict(py)))?;

        // pass arguments as HashMap
        let mut kwargs = HashMap::<&str, i32>::new();
        kwargs.insert(key1, 1);
        fun.call(py, (), Some(kwargs.into_py_dict(py)))?;

        Ok(())
   })
}
```

## Executing existing Python code

If you already have some existing Python code that you need to execute from Rust, the following FAQs can help you select the right PyO3 functionality for your situation:

### Want to access Python APIs? Then use `PyModule::import`.

[`Pymodule::import`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyModule.html#method.import) can
be used to get handle to a Python module from Rust. You can use this to import and use any Python
module available in your environment.

```rust
use pyo3::prelude::*;

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let builtins = PyModule::import(py, "builtins")?;
        let total: i32 = builtins.getattr("sum")?.call1((vec![1, 2, 3],))?.extract()?;
        assert_eq!(total, 6);
        Ok(())
    })
}
```

### Want to run just an expression? Then use `eval`.

[`Python::eval`]({{#PYO3_DOCS_URL}}/pyo3/struct.Python.html#method.eval) is
a method to execute a [Python expression](https://docs.python.org/3.7/reference/expressions.html)
and return the evaluated value as a `&PyAny` object.

```rust
use pyo3::prelude::*;

# fn main() -> Result<(), ()> {
Python::with_gil(|py| {
    let result = py.eval("[i * 10 for i in range(5)]", None, None).map_err(|e| {
        e.print_and_set_sys_last_vars(py);
    })?;
    let res: Vec<i64> = result.extract().unwrap();
    assert_eq!(res, vec![0, 10, 20, 30, 40]);
    Ok(())
})
# }
```

### Want to run statements? Then use `run`.

[`Python::run`] is a method to execute one or more
[Python statements](https://docs.python.org/3.7/reference/simple_stmts.html).
This method returns nothing (like any Python statement), but you can get
access to manipulated objects via the `locals` dict.

You can also use the [`py_run!`] macro, which is a shorthand for [`Python::run`].
Since [`py_run!`] panics on exceptions, we recommend you use this macro only for
quickly testing your Python extensions.

```rust
use pyo3::prelude::*;
use pyo3::{PyCell, py_run};

# fn main() {
#[pyclass]
struct UserData {
    id: u32,
    name: String,
}

#[pymethods]
impl UserData {
    fn as_tuple(&self) -> (u32, String) {
        (self.id, self.name.clone())
    }

    fn __repr__(&self) -> PyResult<String> {
        Ok(format!("User {}(id: {})", self.name, self.id))
    }
}

Python::with_gil(|py| {
    let userdata = UserData {
        id: 34,
        name: "Yu".to_string(),
    };
    let userdata = PyCell::new(py, userdata).unwrap();
    let userdata_as_tuple = (34, "Yu");
    py_run!(py, userdata userdata_as_tuple, r#"
assert repr(userdata) == "User Yu(id: 34)"
assert userdata.as_tuple() == userdata_as_tuple
    "#);
})
# }
```

## You have a Python file or code snippet? Then use `PyModule::from_code`.

[PyModule::from_code]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyModule.html#method.from_code)
can be used to generate a Python module which can then be used just as if it was imported with
`PyModule::import`.

**Warning**: This will compile and execute code. **Never** pass untrusted code
to this function!

```rust
use pyo3::{prelude::*, types::{IntoPyDict, PyModule}};

# fn main() -> PyResult<()> {
Python::with_gil(|py| {
    let activators = PyModule::from_code(py, r#"
def relu(x):
    """see https://en.wikipedia.org/wiki/Rectifier_(neural_networks)"""
    return max(0.0, x)

def leaky_relu(x, slope=0.01):
    return x if x >= 0 else x * slope
    "#, "activators.py", "activators")?;

    let relu_result: f64 = activators.getattr("relu")?.call1((-1.0,))?.extract()?;
    assert_eq!(relu_result, 0.0);

    let kwargs = [("slope", 0.2)].into_py_dict(py);
    let lrelu_result: f64 = activators
        .getattr("leaky_relu")?.call((-1.0,), Some(kwargs))?
        .extract()?;
    assert_eq!(lrelu_result, -0.2);
#    Ok(())
})
# }
```

### Include multiple Python files

You can include a file at compile time by using
[`std::include_str`](https://doc.rust-lang.org/std/macro.include_str.html) macro.

Or you can load a file at runtime by using
[`std::fs::read_to_string`](https://doc.rust-lang.org/std/fs/fn.read_to_string.html) function.

Many Python files can be included and loaded as modules. If one file depends on
another you must preserve correct order while declaring `PyModule`.

Example directory structure:
```text
.
├── Cargo.lock
├── Cargo.toml
├── python_app
│   ├── app.py
│   └── utils
│       └── foo.py
└── src
    └── main.rs
```

`python_app/app.py`:
```python
from utils.foo import bar


def run():
    return bar()
```

`python_app/utils/foo.py`:
```python
def bar():
    return "baz"
```

The example below shows:
* how to include content of `app.py` and `utils/foo.py` into your rust binary
* how to call function `run()` (declared in `app.py`) that needs function
  imported from `utils/foo.py`

`src/main.rs`:
```ignore
use pyo3::prelude::*;

fn main() -> PyResult<()> {
    let py_foo = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/python_app/utils/foo.py"));
    let py_app = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/python_app/app.py"));
    let from_python = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
        PyModule::from_code(py, py_foo, "utils.foo", "utils.foo")?;
        let app: Py<PyAny> = PyModule::from_code(py, py_app, "", "")?
            .getattr("run")?
            .into();
        app.call0(py)
    });

    println!("py: {}", from_python?);
    Ok(())
}
```

The example below shows:
* how to load content of `app.py` at runtime so that it sees its dependencies
  automatically
* how to call function `run()` (declared in `app.py`) that needs function
  imported from `utils/foo.py`

It is recommended to use absolute paths because then your binary can be run
from anywhere as long as your `app.py` is in the expected directory (in this example
that directory is `/usr/share/python_app`).

`src/main.rs`:
```ignore
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::fs;
use std::path::Path;

fn main() -> PyResult<()> {
    let path = Path::new("/usr/share/python_app");
    let py_app = fs::read_to_string(path.join("app.py"))?;
    let from_python = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
        let syspath: &PyList = py.import("sys")?.getattr("path")?.downcast::<PyList>()?;
        syspath.insert(0, &path)?;
        let app: Py<PyAny> = PyModule::from_code(py, &py_app, "", "")?
            .getattr("run")?
            .into();
        app.call0(py)
    });

    println!("py: {}", from_python?);
    Ok(())
}
```


[`Python::run`]: {{#PYO3_DOCS_URL}}/pyo3/struct.Python.html#method.run
[`py_run!`]: {{#PYO3_DOCS_URL}}/pyo3/macro.py_run.html

## Need to use a context manager from Rust?

Use context managers by directly invoking `__enter__` and `__exit__`.

```rust
use pyo3::prelude::*;
use pyo3::types::PyModule;

fn main() {
    Python::with_gil(|py| {
        let custom_manager = PyModule::from_code(py, r#"
class House(object):
    def __init__(self, address):
        self.address = address
    def __enter__(self):
        print(f"Welcome to {self.address}!")
    def __exit__(self, type, value, traceback):
        if type:
            print(f"Sorry you had {type} trouble at {self.address}")
        else:
            print(f"Thank you for visiting {self.address}, come again soon!")

        "#, "house.py", "house").unwrap();

        let house_class = custom_manager.getattr("House").unwrap();
        let house = house_class.call1(("123 Main Street",)).unwrap();

        house.call_method0("__enter__").unwrap();

        let result = py.eval("undefined_variable + 1", None, None);

        // If the eval threw an exception we'll pass it through to the context manager.
        // Otherwise, __exit__  is called with empty arguments (Python "None").
        match result {
            Ok(_) => {
                let none = py.None();
                house.call_method1("__exit__", (&none, &none, &none)).unwrap();
            },
            Err(e) => {
                house.call_method1(
                    "__exit__",
                    (e.get_type(py), e.value(py), e.traceback(py))
                ).unwrap();
            }
        }
    })
}
```
