# Function signatures

The `#[pyfunction]` attribute also accepts parameters to control how the generated Python function accepts arguments. Just like in Python, arguments can be positional-only, keyword-only, or accept either. `*args` lists and `**kwargs` dicts can also be accepted. These parameters also work for `#[pymethods]` which will be introduced in the [Python Classes](../class.md) section of the guide.

Like Python, by default PyO3 accepts all arguments as either positional or keyword arguments. Most arguments are required by default, except for trailing `Option<_>` arguments, which are [implicitly given a default of `None`](#trailing-optional-arguments). This behaviour can be configured by the `#[pyo3(signature = (...))]` option which allows writing a signature in Python syntax.

This section of the guide goes into detail about use of the `#[pyo3(signature = (...))]` option and its related option `#[pyo3(text_signature = "...")]`

## Using `#[pyo3(signature = (...))]`

For example, below is a function that accepts arbitrary keyword arguments (`**kwargs` in Python syntax) and returns the number that was passed:

```rust
use pyo3::prelude::*;
use pyo3::types::PyDict;

#[pyfunction]
#[pyo3(signature = (**kwds))]
fn num_kwds(kwds: Option<&Bound<'_, PyDict>>) -> usize {
    kwds.map_or(0, |dict| dict.len().unwrap_or_default())
}

#[pymodule]
fn module_with_functions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(num_kwds, m)?)
}
```

Just like in Python, the following constructs can be part of the signature::

 * `/`: positional-only arguments separator, each parameter defined before `/` is a positional-only parameter.
 * `*`: var arguments separator, each parameter defined after `*` is a keyword-only parameter.
 * `*args`: "args" is var args. Type of the `args` parameter has to be `&Bound<'_, PyTuple>`.
 * `**kwargs`: "kwargs" receives keyword arguments. The type of the `kwargs` parameter has to be `Option<&Bound<'_, PyDict>>`.
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
        py_args: &Bound<'_, PyTuple>,
        name: &str,
        py_kwargs: Option<&Bound<'_, PyDict>>,
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

Arguments of type `Python` must not be part of the signature:

```rust
# #![allow(dead_code)]
# use pyo3::prelude::*;
#[pyfunction]
#[pyo3(signature = (lambda))]
pub fn simple_python_bound_function(py: Python<'_>, lambda: PyObject) -> PyResult<()> {
    Ok(())
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

<div class="warning">

⚠️ Warning: This behaviour is being phased out 🛠️

The special casing of trailing optional arguments is deprecated. In a future `pyo3` version, arguments of type `Option<..>` will share the same behaviour as other arguments, they are required unless a default is set using `#[pyo3(signature = (...))]`.

This is done to better align the Python and Rust definition of such functions and make it more intuitive to rewrite them from Python in Rust. Specifically `def some_fn(a: int, b: Optional[int]): ...` will not automatically default `b` to `none`, but requires an explicit default if desired, where as in current `pyo3` it is handled the other way around.

During the migration window a `#[pyo3(signature = (...))]` will be required to silence the deprecation warning. After support for trailing optional arguments is fully removed, the signature attribute can be removed if all arguments should be required.
</div>


As a convenience, functions without a `#[pyo3(signature = (...))]` option will treat trailing `Option<T>` arguments as having a default of `None`. In the example below, PyO3 will create `increment` with a signature of `increment(x, amount=None)`.

```rust
#![allow(deprecated)]
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
#         let fun = pyo3::wrap_pyfunction_bound!(increment, py)?;
#
#         let inspect = PyModule::import_bound(py, "inspect")?.getattr("signature")?;
#         let sig: String = inspect
#             .call1((fun,))?
#             .call_method0("__str__")?
#             .extract()?;
#
#         #[cfg(Py_3_8)]  // on 3.7 the signature doesn't render b, upstream bug?
#         assert_eq!(sig, "(x, amount=None)");
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
#         let fun = pyo3::wrap_pyfunction_bound!(increment, py)?;
#
#         let inspect = PyModule::import_bound(py, "inspect")?.getattr("signature")?;
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

## Making the function signature available to Python

The function signature is exposed to Python via the `__text_signature__` attribute. PyO3 automatically generates this for every `#[pyfunction]` and all `#[pymethods]` directly from the Rust function, taking into account any override done with the `#[pyo3(signature = (...))]` option.

This automatic generation can only display the value of default arguments for strings, integers, boolean types, and `None`. Any other default arguments will be displayed as `...`. (`.pyi` type stub files commonly also use `...` for default arguments in the same way.)

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
#         let fun = pyo3::wrap_pyfunction_bound!(add, py)?;
#
#         let doc: String = fun.getattr("__doc__")?.extract()?;
#         assert_eq!(doc, "This function adds two unsigned 64-bit integers.");
#
#         let inspect = PyModule::import_bound(py, "inspect")?.getattr("signature")?;
#         let sig: String = inspect
#             .call1((fun,))?
#             .call_method0("__str__")?
#             .extract()?;
#
#         #[cfg(Py_3_8)]  // on 3.7 the signature doesn't render b, upstream bug?
#         assert_eq!(sig, "(a, b=0, /)");
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
Signature: pyo3_test.add(a, b=0, /)
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
#         let fun = pyo3::wrap_pyfunction_bound!(add, py)?;
#
#         let doc: String = fun.getattr("__doc__")?.extract()?;
#         assert_eq!(doc, "This function adds two unsigned 64-bit integers.");
#
#         let inspect = PyModule::import_bound(py, "inspect")?.getattr("signature")?;
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

PyO3 will include the contents of the annotation unmodified as the `__text_signature__`. Below shows how IPython will now present this (see the default value of 0 for b):

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
#         let fun = pyo3::wrap_pyfunction_bound!(add, py)?;
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
