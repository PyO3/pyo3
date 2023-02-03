# Function signatures

The `#[pyfunction]` attribute also accepts parameters to control how the generated Python function accepts arguments. Just like in Python, arguments can be positional-only, keyword-only, or accept either. `*args` lists and `**kwargs` dicts can also be accepted. These parameters also work for `#[pymethods]` which will be introduced in the [Python Classes](../class.md) section of the guide.

Like Python, by default PyO3 accepts all arguments as either positional or keyword arguments. Most arguments are required by default, except for trailing `Option<_>` arguments, which are [implicitly given a default of `None`](#trailing-optional-arguments). There are two ways to modify this behaviour:
  - The `#[pyo3(signature = (...))]` option which allows writing a signature in Python syntax.
  - Extra arguments directly to `#[pyfunction]`. (See deprecated form)

## Using `#[pyo3(signature = (...))]`

For example, below is a function that accepts arbitrary keyword arguments (`**kwargs` in Python syntax) and returns the number that was passed:

```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyfunction]
#[pyo3(signature = (**kwds))]
fn num_kwds(kwds: Option<&PyDict>) -> usize {
    kwds.map_or(0, |dict| dict.len())
}

#[pymodule]
fn module_with_functions(py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(num_kwds, m)?).unwrap();
    Ok(())
}
```

Just like in Python, the following constructs can be part of the signature::

 * `/`: positional-only arguments separator, each parameter defined before `/` is a positional-only parameter.
 * `*`: var arguments separator, each parameter defined after `*` is a keyword-only parameter.
 * `*args`: "args" is var args. Type of the `args` parameter has to be `&PyTuple`.
 * `**kwargs`: "kwargs" receives keyword arguments. The type of the `kwargs` parameter has to be `Option<&PyDict>`.
 * `arg=Value`: arguments with default value.
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
    #[pyo3(signature = (num=-1))]
    fn new(num: i32) -> Self {
        MyClass { num }
    }

    #[pyo3(signature = (num=10, *py_args, name="Hello", **py_kwargs))]
    fn method(
        &mut self,
        num: i32,
        py_args: &PyTuple,
        name: &str,
        py_kwargs: Option<&PyDict>,
    ) -> String {
        let num_before = self.num;
        self.num = num;
        format!(
            "num={} (was previously={}), py_args={:?}, name={}, py_kwargs={:?} ",
            num, num_before, py_args, name, py_kwargs,
        )
    }

    fn make_change(&mut self, num: i32) -> PyResult<String> {
        self.num = num;
        Ok(format!("num={}", self.num))
    }
}
```
N.B. the position of the `/` and `*` arguments (if included) control the system of handling positional and keyword arguments. In Python:
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

> Note: to use keywords like `struct` as a function argument, use "raw identifier" syntax `r#struct` in both the signature and the function definition:
>
> ```rust
> # #![allow(dead_code)]
> # use pyo3::prelude::*;
> #[pyfunction(signature = (r#struct = "foo"))]
> fn function_with_keyword(r#struct: &str) {
> #     let _ = r#struct;
>     /* ... */
> }
> ```

## Trailing optional arguments

As a convenience, functions without a `#[pyo3(signature = (...))]` option will treat trailing `Option<T>` arguments as having a default of `None`. In the example below, PyO3 will create `increment` with a signature of `increment(x, amount=None)`.

```rust
use pyo3::prelude::*;

/// Returns a copy of `x` increased by `amount`.
///
/// If `amount` is unspecified or `None`, equivalent to `x + 1`.
#[pyfunction]
fn increment(x: u64, amount: Option<u64>) -> u64 {
    x + amount.unwrap_or(1)
}
#
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| {
#         let fun = pyo3::wrap_pyfunction!(increment, py)?;
#
#         let inspect = PyModule::import(py, "inspect")?.getattr("signature")?;
#         let sig: String = inspect
#             .call1((fun,))?
#             .call_method0("__str__")?
#             .extract()?;
#
#         #[cfg(Py_3_8)]  // on 3.7 the signature doesn't render b, upstream bug?
#         assert_eq!(sig, "(x, amount=Ellipsis)");
#
#         Ok(())
#     })
# }
```

To make trailing `Option<T>` arguments required, but still accept `None`, add a `#[pyo3(signature = (...))]` annotation. For the example above, this would be `#[pyo3(signature = (x, amount))]`:

```rust
# use pyo3::prelude::*;
#[pyfunction]
#[pyo3(signature = (x, amount))]
fn increment(x: u64, amount: Option<u64>) -> u64 {
    x + amount.unwrap_or(1)
}
#
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| {
#         let fun = pyo3::wrap_pyfunction!(increment, py)?;
#
#         let inspect = PyModule::import(py, "inspect")?.getattr("signature")?;
#         let sig: String = inspect
#             .call1((fun,))?
#             .call_method0("__str__")?
#             .extract()?;
#
#         #[cfg(Py_3_8)]  // on 3.7 the signature doesn't render b, upstream bug?
#         assert_eq!(sig, "(x, amount)");
#
#         Ok(())
#     })
# }
```

To help avoid confusion, PyO3 requires `#[pyo3(signature = (...))]` when an `Option<T>` argument is surrounded by arguments which aren't `Option<T>`.

## Deprecated form

The `#[pyfunction]` macro can take the argument specification directly, but this method is deprecated in PyO3 0.18 because the `#[pyo3(signature)]` option offers a simpler syntax and better validation.

The `#[pymethods]` macro has an `#[args]` attribute which accepts the deprecated form.

Below are the same examples as above which using the deprecated syntax:

```rust
# #![allow(deprecated)]

use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyfunction(kwds = "**")]
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
# #![allow(deprecated)]
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

    #[args(num = "10", py_args = "*", name = "\"Hello\"", py_kwargs = "**")]
    fn method(
        &mut self,
        num: i32,
        py_args: &PyTuple,
        name: &str,
        py_kwargs: Option<&PyDict>,
    ) -> String {
        let num_before = self.num;
        self.num = num;
        format!(
            "num={} (was previously={}), py_args={:?}, name={}, py_kwargs={:?} ",
            num, num_before, py_args, name, py_kwargs,
        )
    }

    fn make_change(&mut self, num: i32) -> PyResult<String> {
        self.num = num;
        Ok(format!("num={}", self.num))
    }
}
```

## Making the function signature available to Python

The function signature is exposed to Python via the `__text_signature__` attribute. PyO3 automatically generates this for every `#[pyfunction]` and all `#[pymethods]` directly from the Rust function, taking into account any override done with the `#[pyo3(signature = (...))]` option.

This automatic generation has some limitations, which may be improved in the future:
- It will not include the value of default arguments, replacing them all with `...`. (`.pyi` type stub files commonly also use `...` for all default arguments in the same way.)
- Nothing is generated for the `#[new]` method of a `#[pyclass]`.

In cases where the automatically-generated signature needs adjusting, it can [be overridden](#overriding-the-generated-signature) using the `#[pyo3(text_signature)]` option.)

The example below creates a function `add` which accepts two positional-only arguments `a` and `b`, where `b` has a default value of zero.

```rust
use pyo3::prelude::*;

/// This function adds two unsigned 64-bit integers.
#[pyfunction]
#[pyo3(signature = (a, b=0, /))]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
#
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| {
#         let fun = pyo3::wrap_pyfunction!(add, py)?;
#
#         let doc: String = fun.getattr("__doc__")?.extract()?;
#         assert_eq!(doc, "This function adds two unsigned 64-bit integers.");
#
#         let inspect = PyModule::import(py, "inspect")?.getattr("signature")?;
#         let sig: String = inspect
#             .call1((fun,))?
#             .call_method0("__str__")?
#             .extract()?;
#
#         #[cfg(Py_3_8)]  // on 3.7 the signature doesn't render b, upstream bug?
#         assert_eq!(sig, "(a, b=Ellipsis, /)");
#
#         Ok(())
#     })
# }
```

The following IPython output demonstrates how this generated signature will be seen from Python tooling:

```text
>>> pyo3_test.add.__text_signature__
'(a, b=..., /)'
>>> pyo3_test.add?
Signature: pyo3_test.add(a, b=Ellipsis, /)
Docstring: This function adds two unsigned 64-bit integers.
Type:      builtin_function_or_method
```

### Overriding the generated signature

The `#[pyo3(text_signature = "(<some signature>)")]` attribute can be used to override the default generated signature.

In the snippet below, the text signature attribute is used to include the default value of `0` for the argument `b`, instead of the automatically-generated default value of `...`:

```rust
use pyo3::prelude::*;

/// This function adds two unsigned 64-bit integers.
#[pyfunction]
#[pyo3(signature = (a, b=0, /), text_signature = "(a, b=0, /)")]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
#
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| {
#         let fun = pyo3::wrap_pyfunction!(add, py)?;
#
#         let doc: String = fun.getattr("__doc__")?.extract()?;
#         assert_eq!(doc, "This function adds two unsigned 64-bit integers.");
#
#         let inspect = PyModule::import(py, "inspect")?.getattr("signature")?;
#         let sig: String = inspect
#             .call1((fun,))?
#             .call_method0("__str__")?
#             .extract()?;
#         assert_eq!(sig, "(a, b=0, /)");
#
#         Ok(())
#     })
# }
```

PyO3 will include the contents of the annotation unmodified as the `__text_signature`. Below shows how IPython will now present this (see the default value of 0 for b):

```text
>>> pyo3_test.add.__text_signature__
'(a, b=0, /)'
>>> pyo3_test.add?
Signature: pyo3_test.add(a, b=0, /)
Docstring: This function adds two unsigned 64-bit integers.
Type:      builtin_function_or_method
```

If no signature is wanted at all, `#[pyo3(text_signature = None)]` will disable the built-in signature. The snippet below demonstrates use of this:

```rust
use pyo3::prelude::*;

/// This function adds two unsigned 64-bit integers.
#[pyfunction]
#[pyo3(signature = (a, b=0, /), text_signature = None)]
fn add(a: u64, b: u64) -> u64 {
    a + b
}
#
# fn main() -> PyResult<()> {
#     Python::with_gil(|py| {
#         let fun = pyo3::wrap_pyfunction!(add, py)?;
#
#         let doc: String = fun.getattr("__doc__")?.extract()?;
#         assert_eq!(doc, "This function adds two unsigned 64-bit integers.");
#         assert!(fun.getattr("__text_signature__")?.is_none());
#
#         Ok(())
#     })
# }
```

Now the function's `__text_signature__` will be set to `None`, and IPython will not display any signature in the help:

```text
>>> pyo3_test.add.__text_signature__ == None
True
>>> pyo3_test.add?
Docstring: This function adds two unsigned 64-bit integers.
Type:      builtin_function_or_method
```
