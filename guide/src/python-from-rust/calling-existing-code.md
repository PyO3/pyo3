# Executing existing Python code

If you already have some existing Python code that you need to execute from Rust, the following FAQs can help you select the right PyO3 functionality for your situation:

## Want to access Python APIs? Then use `PyModule::import`.

[`PyModule::import`] can be used to get handle to a Python module from Rust. You can use this to import and use any Python
module available in your environment.

```rust
use pyo3::prelude::*;

fn main() -> PyResult<()> {
    Python::attach(|py| {
        let builtins = PyModule::import(py, "builtins")?;
        let total: i32 = builtins
            .getattr("sum")?
            .call1((vec![1, 2, 3],))?
            .extract()?;
        assert_eq!(total, 6);
        Ok(())
    })
}
```

[`PyModule::import`]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyModule.html#method.import

## Want to run just an expression? Then use `eval`.

[`Python::eval`]({{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.eval) is
a method to execute a [Python expression](https://docs.python.org/3/reference/expressions.html)
and return the evaluated value as a `Bound<'py, PyAny>` object.

```rust
use pyo3::prelude::*;
use pyo3::ffi::c_str;

# fn main() -> Result<(), ()> {
Python::attach(|py| {
    let result = py
        .eval(c_str!("[i * 10 for i in range(5)]"), None, None)
        .map_err(|e| {
            e.print_and_set_sys_last_vars(py);
        })?;
    let res: Vec<i64> = result.extract().unwrap();
    assert_eq!(res, vec![0, 10, 20, 30, 40]);
    Ok(())
})
# }
```

## Want to run statements? Then use `run`.

[`Python::run`] is a method to execute one or more
[Python statements](https://docs.python.org/3/reference/simple_stmts.html).
This method returns nothing (like any Python statement), but you can get
access to manipulated objects via the `locals` dict.

You can also use the [`py_run!`] macro, which is a shorthand for [`Python::run`].
Since [`py_run!`] panics on exceptions, we recommend you use this macro only for
quickly testing your Python extensions.

[`Python::run`]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.run

```rust
use pyo3::prelude::*;
use pyo3::py_run;

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

Python::attach(|py| {
    let userdata = UserData {
        id: 34,
        name: "Yu".to_string(),
    };
    let userdata = Py::new(py, userdata).unwrap();
    let userdata_as_tuple = (34, "Yu");
    py_run!(py, userdata userdata_as_tuple, r#"
assert repr(userdata) == "User Yu(id: 34)"
assert userdata.as_tuple() == userdata_as_tuple
    "#);
})
# }
```

## You have a Python file or code snippet? Then use `PyModule::from_code`.

[`PyModule::from_code`]({{#PYO3_DOCS_URL}}/pyo3/types/struct.PyModule.html#method.from_code)
can be used to generate a Python module which can then be used just as if it was imported with
`PyModule::import`.

**Warning**: This will compile and execute code. **Never** pass untrusted code
to this function!

```rust
use pyo3::{prelude::*, types::IntoPyDict};
use pyo3_ffi::c_str;

# fn main() -> PyResult<()> {
Python::attach(|py| {
    let activators = PyModule::from_code(
        py,
        c_str!(r#"
def relu(x):
    """see https://en.wikipedia.org/wiki/Rectifier_(neural_networks)"""
    return max(0.0, x)

def leaky_relu(x, slope=0.01):
    return x if x >= 0 else x * slope
    "#),
        c_str!("activators.py"),
        c_str!("activators"),
    )?;

    let relu_result: f64 = activators.getattr("relu")?.call1((-1.0,))?.extract()?;
    assert_eq!(relu_result, 0.0);

    let kwargs = [("slope", 0.2)].into_py_dict(py)?;
    let lrelu_result: f64 = activators
        .getattr("leaky_relu")?
        .call((-1.0,), Some(&kwargs))?
        .extract()?;
    assert_eq!(lrelu_result, -0.2);
#    Ok(())
})
# }
```

## Want to embed Python in Rust with additional modules?

Python maintains the `sys.modules` dict as a cache of all imported modules.
An import in Python will first attempt to lookup the module from this dict,
and if not present will use various strategies to attempt to locate and load
the module.

The [`append_to_inittab`]({{#PYO3_DOCS_URL}}/pyo3/macro.append_to_inittab.html)
macro can be used to add additional `#[pymodule]` modules to an embedded
Python interpreter. The macro **must** be invoked _before_ initializing Python.

As an example, the below adds the module `foo` to the embedded interpreter:

```rust
use pyo3::prelude::*;
use pyo3::ffi::c_str;

#[pymodule]
mod foo {
    use pyo3::prelude::*;
    
    #[pyfunction]
    fn add_one(x: i64) -> i64 {
        x + 1
    }
}

fn main() -> PyResult<()> {
    pyo3::append_to_inittab!(foo);
    Python::attach(|py| Python::run(py, c_str!("import foo; foo.add_one(6)"), None, None))
}
```

If `append_to_inittab` cannot be used due to constraints in the program,
an alternative is to create a module using [`PyModule::new`]
and insert it manually into `sys.modules`:

```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::ffi::c_str;

#[pyfunction]
pub fn add_one(x: i64) -> i64 {
    x + 1
}

fn main() -> PyResult<()> {
    Python::attach(|py| {
        // Create new module
        let foo_module = PyModule::new(py, "foo")?;
        foo_module.add_function(wrap_pyfunction!(add_one, &foo_module)?)?;

        // Import and get sys.modules
        let sys = PyModule::import(py, "sys")?;
        let py_modules: Bound<'_, PyDict> = sys.getattr("modules")?.cast_into()?;

        // Insert foo into sys.modules
        py_modules.set_item("foo", foo_module)?;

        // Now we can import + run our python code
        Python::run(py, c_str!("import foo; foo.add_one(6)"), None, None)
    })
}
```

## Include multiple Python files

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
│   ├── app.py
│   └── utils
│       └── foo.py
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
```rust,ignore
use pyo3::prelude::*;
use pyo3_ffi::c_str;

fn main() -> PyResult<()> {
    let py_foo = c_str!(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/python_app/utils/foo.py"
    )));
    let py_app = c_str!(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/python_app/app.py")));
    let from_python = Python::attach(|py| -> PyResult<Py<PyAny>> {
        PyModule::from_code(py, py_foo, c_str!("foo.py"), c_str!("utils.foo"))?;
        let app: Py<PyAny> = PyModule::from_code(py, py_app, c_str!("app.py"), c_str!(""))?
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
```rust,no_run
use pyo3::prelude::*;
use pyo3::types::PyList;
use pyo3_ffi::c_str;
use std::fs;
use std::path::Path;
use std::ffi::CString;

fn main() -> PyResult<()> {
    let path = Path::new("/usr/share/python_app");
    let py_app = CString::new(fs::read_to_string(path.join("app.py"))?)?;
    let from_python = Python::attach(|py| -> PyResult<Py<PyAny>> {
        let syspath = py
            .import("sys")?
            .getattr("path")?
            .cast_into::<PyList>()?;
        syspath.insert(0, path)?;
        let app: Py<PyAny> = PyModule::from_code(py, py_app.as_c_str(), c_str!("app.py"), c_str!(""))?
            .getattr("run")?
            .into();
        app.call0(py)
    });

    println!("py: {}", from_python?);
    Ok(())
}
```


[`Python::run`]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html#method.run
[`py_run!`]: {{#PYO3_DOCS_URL}}/pyo3/macro.py_run.html

## Need to use a context manager from Rust?

Use context managers by directly invoking `__enter__` and `__exit__`.

```rust
use pyo3::prelude::*;
use pyo3::ffi::c_str;

fn main() {
    Python::attach(|py| {
        let custom_manager = PyModule::from_code(
            py,
            c_str!(r#"
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

        "#),
            c_str!("house.py"),
            c_str!("house"),
        )
        .unwrap();

        let house_class = custom_manager.getattr("House").unwrap();
        let house = house_class.call1(("123 Main Street",)).unwrap();

        house.call_method0("__enter__").unwrap();

        let result = py.eval(c_str!("undefined_variable + 1"), None, None);

        // If the eval threw an exception we'll pass it through to the context manager.
        // Otherwise, __exit__  is called with empty arguments (Python "None").
        match result {
            Ok(_) => {
                let none = py.None();
                house
                    .call_method1("__exit__", (&none, &none, &none))
                    .unwrap();
            }
            Err(e) => {
                house
                    .call_method1(
                        "__exit__",
                        (
                            e.get_type(py),
                            e.value(py),
                            e.traceback(py),
                        ),
                    )
                    .unwrap();
            }
        }
    })
}
```

## Handling system signals/interrupts (Ctrl-C)

The best way to handle system signals when running Rust code is to periodically call `Python::check_signals` to handle any signals captured by Python's signal handler. See also [the FAQ entry](../faq.md#ctrl-c-doesnt-do-anything-while-my-rust-code-is-executing).

Alternatively, set Python's `signal` module to take the default action for a signal:

```rust
use pyo3::prelude::*;

# fn main() -> PyResult<()> {
Python::attach(|py| -> PyResult<()> {
    let signal = py.import("signal")?;
    // Set SIGINT to have the default action
    signal
        .getattr("signal")?
        .call1((signal.getattr("SIGINT")?, signal.getattr("SIG_DFL")?))?;
    Ok(())
})
# }
```


[`PyModule::new`]: {{#PYO3_DOCS_URL}}/pyo3/types/struct.PyModule.html#method.new
