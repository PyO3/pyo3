# Calling Python in Rust code

These APIs work from Rust whenever you have a `Python` object handy, whether
PyO3 is built for an extension module or not.

## Want to run just an expression? Then use `eval`.

[`Python::eval`](https://pyo3.rs/master/doc/pyo3/struct.Python.html#method.eval) is
a method to execute a [Python expression](https://docs.python.org/3.7/reference/expressions.html)
and return the evaluated value as a `&PyAny` object.

```rust
use pyo3::prelude::*;
use pyo3::types::IntoPyDict;

fn main() -> Result<(), ()> {
    let gil = Python::acquire_gil();
    let py = gil.python();
    let result = py.eval("[i * 10 for i in range(5)]", None, None).map_err(|e| {
        e.print_and_set_sys_last_vars(py);
    })?;
    let res: Vec<i64> = result.extract().unwrap();
    assert_eq!(res, vec![0, 10, 20, 30, 40]);
    Ok(())
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
let gil = Python::acquire_gil();
let py = gil.python();
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
# }
```

## You have a Python file or Python function? Then use `PyModule`.

[PyModule](https://pyo3.rs/master/doc/pyo3/types/struct.PyModule.html) also can
execute Python code by calling its methods.

```rust
use pyo3::{prelude::*, types::{IntoPyDict, PyModule}};
#  fn main() -> PyResult<()> {
let gil = Python::acquire_gil();
let py = gil.python();
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
# Ok(()) }
```

[`Python::run`]: https://pyo3.rs/master/doc/pyo3/struct.Python.html#method.run
[`py_run!`]: https://pyo3.rs/master/doc/pyo3/macro.py_run.html
