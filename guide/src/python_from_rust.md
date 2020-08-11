# Calling Python in Rust code

These APIs work from Rust whenever you have a `Python` object handy, whether
PyO3 is built for an extension module or not.

## Want to access Python APIs? Then use `PyModule::import`.

[`Pymodule::import`](https://docs.rs/pyo3/latest/pyo3/types/struct.PyModule.html#method.import) can
be used to get handle to a Python module from Rust. You can use this to import and use any Python
module available in your environment.

```rust
use pyo3::prelude::*;

fn main() -> PyResult<()> {
    Python::with_gil(|py| {
        let builtins = PyModule::import(py, "builtins")?;
        let total: i32 = builtins.call1("sum", (vec![1, 2, 3],))?.extract()?;
        assert_eq!(total, 6);
        Ok(())
    })
}
```

## Want to run just an expression? Then use `eval`.

[`Python::eval`](https://docs.rs/pyo3/latest/pyo3/struct.Python.html#method.eval) is
a method to execute a [Python expression](https://docs.python.org/3.7/reference/expressions.html)
and return the evaluated value as a `&PyAny` object.

```rust
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

fn main() -> Result<(), ()> {
    Python::with_gil(|py| {
        let result = py.eval("[i * 10 for i in range(5)]", None, None).map_err(|e| {
            e.print_and_set_sys_last_vars(py);
        })?;
        let res: Vec<i64> = result.extract().unwrap();
        assert_eq!(res, vec![0, 10, 20, 30, 40]);
        Ok(())
    })
}
```

## Want to run statements? Then use `run`.

[`Python::run`] is a method to execute one or more
[Python statements](https://docs.python.org/3.7/reference/simple_stmts.html).
This method returns nothing (like any Python statement), but you can get
access to manipulated objects via the `locals` dict.

You can also use the [`py_run!`] macro, which is a shorthand for [`Python::run`].
Since [`py_run!`] panics on exceptions, we recommend you use this macro only for
quickly testing your Python extensions.

```rust
use pyo3::prelude::*;
use pyo3::{PyCell, PyObjectProtocol, py_run};
#  fn main() {
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
}
#[pyproto]
impl PyObjectProtocol for UserData {
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

## You have a Python file or Python function? Then use `PyModule::from_code`.

[PyModule::from_code](https://docs.rs/pyo3/latest/pyo3/types/struct.PyModule.html#method.from_code)
can be used to generate a Python module which can then be used just as if it was imported with
`PyModule::import`.

```rust
use pyo3::{prelude::*, types::{IntoPyDict, PyModule}};
#  fn main() -> PyResult<()> {
Python::with_gil(|py| {
    let activators = PyModule::from_code(py, r#"
def relu(x):
    """see https://en.wikipedia.org/wiki/Rectifier_(neural_networks)"""
    return max(0.0, x)

def leaky_relu(x, slope=0.01):
    return x if x >= 0 else x * slope
    "#, "activators.py", "activators")?;

    let relu_result: f64 = activators.call1("relu", (-1.0,))?.extract()?;
    assert_eq!(relu_result, 0.0);

    let kwargs = [("slope", 0.2)].into_py_dict(py);
    let lrelu_result: f64 = activators
        .call("leaky_relu", (-1.0,), Some(kwargs))?
        .extract()?;
    assert_eq!(lrelu_result, -0.2);
#    Ok(())
})
# }
```

[`Python::run`]: https://docs.rs/pyo3/latest/pyo3/struct.Python.html#method.run
[`py_run!`]: https://docs.rs/pyo3/latest/pyo3/macro.py_run.html
