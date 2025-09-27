# Emulating numeric types

At this point we have a `Number` class that we can't actually do any math on!

Before proceeding, we should think about how we want to handle overflows. There are three obvious solutions:
- We can have infinite precision just like Python's `int`. However that would be quite boring - we'd
 be reinventing the wheel.
- We can raise exceptions whenever `Number` overflows, but that makes the API painful to use.
- We can wrap around the boundary of `i32`. This is the approach we'll take here. To do that we'll just forward to `i32`'s
 `wrapping_*` methods.

### Fixing our constructor

Let's address the first overflow, in `Number`'s constructor:

```python
from my_module import Number

n = Number(1 << 1337)
```

```text
Traceback (most recent call last):
  File "example.py", line 3, in <module>
    n = Number(1 << 1337)
OverflowError: Python int too large to convert to C long
```

Instead of relying on the default [`FromPyObject`] extraction to parse arguments, we can specify our
own extraction function, using the `#[pyo3(from_py_with = ...)]` attribute. Unfortunately PyO3
doesn't provide a way to wrap Python integers out of the box, but we can do a Python call to mask it
and cast it to an `i32`.

```rust,no_run
# #![allow(dead_code)]
use pyo3::prelude::*;

fn wrap(obj: &Bound<'_, PyAny>) -> PyResult<i32> {
    let val = obj.call_method1("__and__", (0xFFFFFFFF_u32,))?;
    let val: u32 = val.extract()?;
    //     👇 This intentionally overflows!
    Ok(val as i32)
}
```
We also add documentation, via `///` comments, which are visible to Python users.

```rust,no_run
# #![allow(dead_code)]
use pyo3::prelude::*;

fn wrap(obj: &Bound<'_, PyAny>) -> PyResult<i32> {
    let val = obj.call_method1("__and__", (0xFFFFFFFF_u32,))?;
    let val: u32 = val.extract()?;
    Ok(val as i32)
}

/// Did you ever hear the tragedy of Darth Signed The Overfloweth? I thought not.
/// It's not a story C would tell you. It's a Rust legend.
#[pyclass(module = "my_module")]
struct Number(i32);

#[pymethods]
impl Number {
    #[new]
    fn new(#[pyo3(from_py_with = wrap)] value: i32) -> Self {
        Self(value)
    }
}
```


With that out of the way, let's implement some operators:
```rust,no_run
use pyo3::exceptions::{PyZeroDivisionError, PyValueError};

# use pyo3::prelude::*;
#
# #[pyclass]
# struct Number(i32);
#
#[pymethods]
impl Number {
    fn __add__(&self, other: &Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }

    fn __sub__(&self, other: &Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }

    fn __mul__(&self, other: &Self) -> Self {
        Self(self.0.wrapping_mul(other.0))
    }

    fn __truediv__(&self, other: &Self) -> PyResult<Self> {
        match self.0.checked_div(other.0) {
            Some(i) => Ok(Self(i)),
            None => Err(PyZeroDivisionError::new_err("division by zero")),
        }
    }

    fn __floordiv__(&self, other: &Self) -> PyResult<Self> {
        match self.0.checked_div(other.0) {
            Some(i) => Ok(Self(i)),
            None => Err(PyZeroDivisionError::new_err("division by zero")),
        }
    }

    fn __rshift__(&self, other: &Self) -> PyResult<Self> {
        match other.0.try_into() {
            Ok(rhs) => Ok(Self(self.0.wrapping_shr(rhs))),
            Err(_) => Err(PyValueError::new_err("negative shift count")),
        }
    }

    fn __lshift__(&self, other: &Self) -> PyResult<Self> {
        match other.0.try_into() {
            Ok(rhs) => Ok(Self(self.0.wrapping_shl(rhs))),
            Err(_) => Err(PyValueError::new_err("negative shift count")),
        }
    }
}
```

### Unary arithmetic operations

```rust,no_run
# use pyo3::prelude::*;
#
# #[pyclass]
# struct Number(i32);
#
#[pymethods]
impl Number {
    fn __pos__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __neg__(&self) -> Self {
        Self(-self.0)
    }

    fn __abs__(&self) -> Self {
        Self(self.0.abs())
    }

    fn __invert__(&self) -> Self {
        Self(!self.0)
    }
}
```

### Support for the `complex()`, `int()` and `float()` built-in functions.

```rust,no_run
# use pyo3::prelude::*;
#
# #[pyclass]
# struct Number(i32);
#
use pyo3::types::PyComplex;

#[pymethods]
impl Number {
    fn __int__(&self) -> i32 {
        self.0
    }

    fn __float__(&self) -> f64 {
        self.0 as f64
    }

    fn __complex__<'py>(&self, py: Python<'py>) -> Bound<'py, PyComplex> {
        PyComplex::from_doubles(py, self.0 as f64, 0.0)
    }
}
```

We do not implement the in-place operations like `__iadd__` because we do not wish to mutate `Number`.
Similarly we're not interested in supporting operations with different types, so we do not implement
 the reflected operations like `__radd__` either.

Now Python can use our `Number` class:

```python
from my_module import Number

def hash_djb2(s: str):
	'''
	A version of Daniel J. Bernstein's djb2 string hashing algorithm
	Like many hashing algorithms, it relies on integer wrapping.
	'''

	n = Number(0)
	five = Number(5)

	for x in s:
		n = Number(ord(x)) + ((n << five) - n)
	return n

assert hash_djb2('l50_50') == Number(-1152549421)
```

### Final code

```rust
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use pyo3::exceptions::{PyValueError, PyZeroDivisionError};
use pyo3::prelude::*;
use pyo3::class::basic::CompareOp;
use pyo3::types::{PyComplex, PyString};

fn wrap(obj: &Bound<'_, PyAny>) -> PyResult<i32> {
    let val = obj.call_method1("__and__", (0xFFFFFFFF_u32,))?;
    let val: u32 = val.extract()?;
    Ok(val as i32)
}
/// Did you ever hear the tragedy of Darth Signed The Overfloweth? I thought not.
/// It's not a story C would tell you. It's a Rust legend.
#[pyclass(module = "my_module")]
struct Number(i32);

#[pymethods]
impl Number {
    #[new]
    fn new(#[pyo3(from_py_with = wrap)] value: i32) -> Self {
        Self(value)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
       // Get the class name dynamically in case `Number` is subclassed
       let class_name: Bound<'_, PyString> = slf.get_type().qualname()?;
        Ok(format!("{}({})", class_name, slf.borrow().0))
    }

    fn __str__(&self) -> String {
        self.0.to_string()
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish()
    }

    fn __richcmp__(&self, other: &Self, op: CompareOp) -> PyResult<bool> {
        match op {
            CompareOp::Lt => Ok(self.0 < other.0),
            CompareOp::Le => Ok(self.0 <= other.0),
            CompareOp::Eq => Ok(self.0 == other.0),
            CompareOp::Ne => Ok(self.0 != other.0),
            CompareOp::Gt => Ok(self.0 > other.0),
            CompareOp::Ge => Ok(self.0 >= other.0),
        }
    }

    fn __bool__(&self) -> bool {
        self.0 != 0
    }

    fn __add__(&self, other: &Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }

    fn __sub__(&self, other: &Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }

    fn __mul__(&self, other: &Self) -> Self {
        Self(self.0.wrapping_mul(other.0))
    }

    fn __truediv__(&self, other: &Self) -> PyResult<Self> {
        match self.0.checked_div(other.0) {
            Some(i) => Ok(Self(i)),
            None => Err(PyZeroDivisionError::new_err("division by zero")),
        }
    }

    fn __floordiv__(&self, other: &Self) -> PyResult<Self> {
        match self.0.checked_div(other.0) {
            Some(i) => Ok(Self(i)),
            None => Err(PyZeroDivisionError::new_err("division by zero")),
        }
    }

    fn __rshift__(&self, other: &Self) -> PyResult<Self> {
        match other.0.try_into() {
            Ok(rhs) => Ok(Self(self.0.wrapping_shr(rhs))),
            Err(_) => Err(PyValueError::new_err("negative shift count")),
        }
    }

    fn __lshift__(&self, other: &Self) -> PyResult<Self> {
        match other.0.try_into() {
            Ok(rhs) => Ok(Self(self.0.wrapping_shl(rhs))),
            Err(_) => Err(PyValueError::new_err("negative shift count")),
        }
    }

    fn __xor__(&self, other: &Self) -> Self {
        Self(self.0 ^ other.0)
    }

    fn __or__(&self, other: &Self) -> Self {
        Self(self.0 | other.0)
    }

    fn __and__(&self, other: &Self) -> Self {
        Self(self.0 & other.0)
    }

    fn __int__(&self) -> i32 {
        self.0
    }

    fn __float__(&self) -> f64 {
        self.0 as f64
    }

    fn __complex__<'py>(&self, py: Python<'py>) -> Bound<'py, PyComplex> {
        PyComplex::from_doubles(py, self.0 as f64, 0.0)
    }
}

#[pymodule]
mod my_module {
    #[pymodule_export]
    use super::Number;
}
# const SCRIPT: &'static std::ffi::CStr = pyo3::ffi::c_str!(r#"
# def hash_djb2(s: str):
#     n = Number(0)
#     five = Number(5)
#
#     for x in s:
#         n = Number(ord(x)) + ((n << five) - n)
#     return n
#
# assert hash_djb2('l50_50') == Number(-1152549421)
# assert hash_djb2('logo') == Number(3327403)
# assert hash_djb2('horizon') == Number(1097468315)
#
#
# assert Number(2) + Number(2) == Number(4)
# assert Number(2) + Number(2) != Number(5)
#
# assert Number(13) - Number(7) == Number(6)
# assert Number(13) - Number(-7) == Number(20)
#
# assert Number(13) / Number(7) == Number(1)
# assert Number(13) // Number(7) == Number(1)
#
# assert Number(13) * Number(7) == Number(13*7)
#
# assert Number(13) > Number(7)
# assert Number(13) < Number(20)
# assert Number(13) == Number(13)
# assert Number(13) >= Number(7)
# assert Number(13) <= Number(20)
# assert Number(13) == Number(13)
#
#
# assert (True if Number(1) else False)
# assert (False if Number(0) else True)
#
#
# assert int(Number(13)) == 13
# assert float(Number(13)) == 13
# assert Number.__doc__ == "Did you ever hear the tragedy of Darth Signed The Overfloweth? I thought not.\nIt's not a story C would tell you. It's a Rust legend."
# assert Number(12345234523452) == Number(1498514748)
# try:
#     import inspect
#     assert inspect.signature(Number).__str__() == '(value)'
# except ValueError:
#     # Not supported with `abi3` before Python 3.10
#     pass
# assert Number(1337).__str__() == '1337'
# assert Number(1337).__repr__() == 'Number(1337)'
"#);

#
# use pyo3::PyTypeInfo;
#
# fn main() -> PyResult<()> {
#     Python::attach(|py| -> PyResult<()> {
#         let globals = PyModule::import(py, "__main__")?.dict();
#         globals.set_item("Number", Number::type_object(py))?;
#
#         py.run(SCRIPT, Some(&globals), None)?;
#         Ok(())
#     })
# }
```

## Appendix: Writing some unsafe code

At the beginning of this chapter we said that PyO3 doesn't provide a way to wrap Python integers out
of the box but that's a half truth. There's not a PyO3 API for it, but there's a Python C API
function that does:

```c
unsigned long PyLong_AsUnsignedLongMask(PyObject *obj)
```

We can call this function from Rust by using [`pyo3::ffi::PyLong_AsUnsignedLongMask`]. This is an *unsafe*
function, which means we have to use an unsafe block to call it and take responsibility for upholding
the contracts of this function. Let's review those contracts:
- We must be attached to the interpreter. If we're not, calling this function causes a data race.
- The pointer must be valid, i.e. it must be properly aligned and point to a valid Python object.

Let's create that helper function. The signature has to be `fn(&Bound<'_, PyAny>) -> PyResult<T>`.
- `&Bound<'_, PyAny>` represents a checked bound reference, so the pointer derived from it is valid (and not null).
- Whenever we have bound references to Python objects in scope, it is guaranteed that we're attached to the interpreter. This reference is also where we can get a [`Python`] token to use in our call to [`PyErr::take`].

```rust,no_run
# #![allow(dead_code)]
use std::ffi::c_ulong;
use pyo3::prelude::*;
use pyo3::ffi;

fn wrap(obj: &Bound<'_, PyAny>) -> Result<i32, PyErr> {
    let py: Python<'_> = obj.py();

    unsafe {
        let ptr = obj.as_ptr();

        let ret: c_ulong = ffi::PyLong_AsUnsignedLongMask(ptr);
        if ret == c_ulong::MAX {
            if let Some(err) = PyErr::take(py) {
                return Err(err);
            }
        }

        Ok(ret as i32)
    }
}
```

[`PyErr::take`]: {{#PYO3_DOCS_URL}}/pyo3/prelude/struct.PyErr.html#method.take
[`Python`]: {{#PYO3_DOCS_URL}}/pyo3/marker/struct.Python.html
[`FromPyObject`]: {{#PYO3_DOCS_URL}}/pyo3/conversion/trait.FromPyObject.html
[`pyo3::ffi::PyLong_AsUnsignedLongMask`]: {{#PYO3_DOCS_URL}}/pyo3/ffi/fn.PyLong_AsUnsignedLongMask.html
