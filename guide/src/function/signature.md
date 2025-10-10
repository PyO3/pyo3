# Function signatures

The `#[pyfunction]` attribute also accepts parameters to control how the generated Python function accepts arguments. Just like in Python, arguments can be positional-only, keyword-only, or accept either. `*args` lists and `**kwargs` dicts can also be accepted. These parameters also work for `#[pymethods]` which will be introduced in the [Python Classes](../class.md) section of the guide.

Like Python, by default PyO3 accepts all arguments as either positional or keyword arguments. All arguments are required by default. This behaviour can be configured by the `#[pyo3(signature = (...))]` option which allows writing a signature in Python syntax.

This section of the guide goes into detail about use of the `#[pyo3(signature = (...))]` option and its related option `#[pyo3(text_signature = "...")]`

## Using `#[pyo3(signature = (...))]`

For example, below is a function that accepts arbitrary keyword arguments (`**kwargs` in Python syntax) and returns the number that was passed:

```rust,no_run
#[pyo3::pymodule]
mod module_with_functions {
    use pyo3::prelude::*;
    use pyo3::types::PyDict;

    #[pyfunction]
    #[pyo3(signature = (**kwds))]
    fn num_kwds(kwds: Option<&Bound<'_, PyDict>>) -> usize {
        kwds.map_or(0, |dict| dict.len())
    }
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
```rust,no_run
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

```rust,no_run
# #![allow(dead_code)]
# use pyo3::prelude::*;
#[pyfunction]
#[pyo3(signature = (lambda))]
pub fn simple_python_bound_function(py: Python<'_>, lambda: Py<PyAny>) -> PyResult<()> {
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
> ```rust,no_run
> # #![allow(dead_code)]
> # use pyo3::prelude::*;
> #[pyfunction(signature = (r#struct = "foo"))]
> fn function_with_keyword(r#struct: &str) {
> #     let _ = r#struct;
>     /* ... */
> }
> ```

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
#     Python::attach(|py| {
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
#     Python::attach(|py| {
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
#     Python::attach(|py| {
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

### Type annotations in the signature

When the `experimental-inspect` Cargo feature is enabled, the `signature` attribute can also contain type hints:
```rust
# #[cfg(feature = "experimental-inspect")] {
use pyo3::prelude::*;

#[pymodule]
pub mod example {
   use pyo3::prelude::*;

   #[pyfunction]
   #[pyo3(signature = (arg: "list[int]") -> "list[int]")]
   fn list_of_int_identity(arg: Bound<'_, PyAny>) -> Bound<'_, PyAny> {
      arg
   }
}
# }
```

It enables the [work-in-progress capacity of PyO3 to autogenerate type stubs](../type-stub.md) to generate a file with the correct type hints:
```python
def list_of_int_identity(arg: list[int]) -> list[int]: ...
```
instead of the generic:
```python
import typing

def list_of_int_identity(arg: typing.Any) -> typing.Any: ...
```

Note that currently type annotations must be written as Rust strings.
