# Function signatures

The `#[pyfunction]` attribute also accepts parameters to control how the generated Python function accepts arguments. Just like in Python, arguments can be positional-only, keyword-only, or accept either. `*args` lists and `**kwargs` dicts can also be accepted. These parameters also work for `#[pymethods]` which will be introduced in the [Python Classes](../class.md) section of the guide.

Like Python, by default PyO3 accepts all arguments as either positional or keyword arguments. The extra arguments to `#[pyfunction]` modify this behaviour. For example, below is a function that accepts arbitrary keyword arguments (`**kwargs` in Python syntax) and returns the number that was passed:

```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyfunction(kwds="**")]
fn num_kwds(kwds: Option<&PyDict>) -> usize {
    kwds.map_or(0, |dict| dict.len())
}

#[pymodule]
fn module_with_functions(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(num_kwds, m)?).unwrap();
    Ok(())
}
```

The following parameters can be passed to the `#[pyfunction]` attribute:

 * `"/"`: positional-only arguments separator, each parameter defined before `"/"` is a
   positional-only parameter.
   Corresponds to python's `def meth(arg1, arg2, ..., /, argN..)`.
 * `"*"`: var arguments separator, each parameter defined after `"*"` is a keyword-only parameter.
   Corresponds to python's `def meth(*, arg1.., arg2=..)`.
 * `args="*"`: "args" is var args, corresponds to Python's `def meth(*args)`. Type of the `args`
   parameter has to be `&PyTuple`.
 * `kwargs="**"`: "kwargs" receives keyword arguments, corresponds to Python's `def meth(**kwargs)`.
   The type of the `kwargs` parameter has to be `Option<&PyDict>`.
 * `arg="Value"`: arguments with default value. Corresponds to Python's `def meth(arg=Value)`.
   If the `arg` argument is defined after var arguments, it is treated as a keyword-only argument.
   Note that `Value` has to be valid rust code, PyO3 just inserts it into the generated
   code unmodified.

Example:
```rust
# use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
#
# #[pyclass]
# struct MyClass {
#     num: i32,
# }
#[pymethods]
impl MyClass {
    #[new]
    #[args(num = "-1")]
    fn new(num: i32) -> Self {
        MyClass { num }
    }

    #[args(
        num = "10",
        py_args = "*",
        name = "\"Hello\"",
        py_kwargs = "**"
    )]
    fn method(
        &mut self,
        num: i32,
        name: &str,
        py_args: &PyTuple,
        py_kwargs: Option<&PyDict>,
    ) -> PyResult<String> {
        self.num = num;
        Ok(format!(
            "py_args={:?}, py_kwargs={:?}, name={}, num={}",
            py_args, py_kwargs, name, self.num
        ))
    }

    fn make_change(&mut self, num: i32) -> PyResult<String> {
        self.num = num;
        Ok(format!("num={}", self.num))
    }
}
```
N.B. the position of the `"/"` and `"*"` arguments (if included) control the system of handling positional and keyword arguments. In Python:
```python
import mymodule

mc = mymodule.MyClass()
print(mc.method(44, False, "World", 666, x=44, y=55))
print(mc.method(num=-1, name="World"))
print(mc.make_change(44, False))
```
Produces output:
```text
py_args=('World', 666), py_kwargs=Some({'x': 44, 'y': 55}), name=Hello, num=44
py_args=(), py_kwargs=None, name=World, num=-1
num=44
num=-1
```
