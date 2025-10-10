# Basic object customization

Recall the `Number` class from the previous chapter:

```rust,no_run
# #![allow(dead_code)]
# fn main() {}
use pyo3::prelude::*;

#[pyclass]
struct Number(i32);

#[pymethods]
impl Number {
    #[new]
    fn new(value: i32) -> Self {
        Self(value)
    }
}

#[pymodule]
mod my_module {
    #[pymodule_export]
    use super::Number;
}
```

At this point Python code can import the module, access the class and create class instances - but
nothing else.

```python
from my_module import Number

n = Number(5)
print(n)
```

```text
<builtins.Number object at 0x000002B4D185D7D0>
```

### String representations

It can't even print an user-readable representation of itself! We can fix that by defining the
`__repr__` and `__str__` methods inside a `#[pymethods]` block. We do this by accessing the value
contained inside `Number`.

```rust,no_run
# use pyo3::prelude::*;
#
# #[pyclass]
# struct Number(i32);
#
#[pymethods]
impl Number {
    // For `__repr__` we want to return a string that Python code could use to recreate
    // the `Number`, like `Number(5)` for example.
    fn __repr__(&self) -> String {
        // We use the `format!` macro to create a string. Its first argument is a
        // format string, followed by any number of parameters which replace the
        // `{}`'s in the format string.
        //
        //                       👇 Tuple field access in Rust uses a dot
        format!("Number({})", self.0)
    }
    // `__str__` is generally used to create an "informal" representation, so we
    // just forward to `i32`'s `ToString` trait implementation to print a bare number.
    fn __str__(&self) -> String {
        self.0.to_string()
    }
}
```

To automatically generate the `__str__` implementation using a `Display` trait implementation, pass the `str` argument to `pyclass`.

```rust,no_run
# use std::fmt::{Display, Formatter};
# use pyo3::prelude::*;
#
# #[allow(dead_code)]
#[pyclass(str)]
struct Coordinate {
    x: i32,
    y: i32,
    z: i32,
}

impl Display for Coordinate {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
```

For convenience, a shorthand format string can be passed to `str` as `str="<format string>"` for **structs only**.  It expands and is passed into the `format!` macro in the following ways:

* `"{x}"` -> `"{}", self.x`
* `"{0}"` -> `"{}", self.0`
* `"{x:?}"` -> `"{:?}", self.x`

*Note: Depending upon the format string you use, this may require implementation of the `Display` or `Debug` traits for the given Rust types.*
*Note: the pyclass args `name` and `rename_all` are incompatible with the shorthand format string and will raise a compile time error.*

```rust,no_run
# use pyo3::prelude::*;
#
# #[allow(dead_code)]
#[pyclass(str="({x}, {y}, {z})")]
struct Coordinate {
    x: i32,
    y: i32,
    z: i32,
}
```

#### Accessing the class name

In the `__repr__`, we used a hard-coded class name. This is sometimes not ideal,
because if the class is subclassed in Python, we would like the repr to reflect
the subclass name. This is typically done in Python code by accessing
`self.__class__.__name__`. In order to be able to access the Python type information
*and* the Rust struct, we need to use a `Bound` as the `self` argument.

```rust,no_run
# use pyo3::prelude::*;
# use pyo3::types::PyString;
#
# #[allow(dead_code)]
# #[pyclass]
# struct Number(i32);
#
#[pymethods]
impl Number {
    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
        // This is the equivalent of `self.__class__.__name__` in Python.
        let class_name: Bound<'_, PyString> = slf.get_type().qualname()?;
        // To access fields of the Rust struct, we need to borrow from the Bound object.
        Ok(format!("{}({})", class_name, slf.borrow().0))
    }
}
```

### Hashing


Let's also implement hashing. We'll just hash the `i32`. For that we need a [`Hasher`]. The one
provided by `std` is [`DefaultHasher`], which uses the [SipHash] algorithm.

```rust,no_run
use std::collections::hash_map::DefaultHasher;

// Required to call the `.hash` and `.finish` methods, which are defined on traits.
use std::hash::{Hash, Hasher};

# use pyo3::prelude::*;
#
# #[allow(dead_code)]
# #[pyclass]
# struct Number(i32);
#
#[pymethods]
impl Number {
    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish()
    }
}
```
To implement `__hash__` using the Rust [`Hash`] trait implementation, the `hash` option can be used.
This option is only available for `frozen` classes to prevent accidental hash changes from mutating the object. If you need
an `__hash__` implementation for a mutable class, use the manual method from above. This option also requires `eq`: According to the
[Python docs](https://docs.python.org/3/reference/datamodel.html#object.__hash__) "If a class does not define an `__eq__()`
method it should not define a `__hash__()` operation either"
```rust,no_run
# use pyo3::prelude::*;
#
# #[allow(dead_code)]
#[pyclass(frozen, eq, hash)]
#[derive(PartialEq, Hash)]
struct Number(i32);
```


> **Note**: When implementing `__hash__` and comparisons, it is important that the following property holds:
>
> ```text
> k1 == k2 -> hash(k1) == hash(k2)
> ```
>
> In other words, if two keys are equal, their hashes must also be equal. In addition you must take
> care that your classes' hash doesn't change during its lifetime. In this tutorial we do that by not
> letting Python code change our `Number` class. In other words, it is immutable.
>
> By default, all `#[pyclass]` types have a default hash implementation from Python.
> Types which should not be hashable can override this by setting `__hash__` to None.
> This is the same mechanism as for a pure-Python class. This is done like so:
>
> ```rust,no_run
> # use pyo3::prelude::*;
> #[pyclass]
> struct NotHashable {}
>
> #[pymethods]
> impl NotHashable {
>     #[classattr]
>     const __hash__: Option<Py<PyAny>> = None;
> }
> ```

### Comparisons

PyO3 supports the usual magic comparison methods available in Python such as `__eq__`, `__lt__`
and so on. It is also possible to support all six operations at once with `__richcmp__`.
This method will be called with a value of `CompareOp` depending on the operation.

```rust,no_run
use pyo3::class::basic::CompareOp;

# use pyo3::prelude::*;
#
# #[allow(dead_code)]
# #[pyclass]
# struct Number(i32);
#
#[pymethods]
impl Number {
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
}
```

If you obtain the result by comparing two Rust values, as in this example, you
can take a shortcut using `CompareOp::matches`:

```rust,no_run
use pyo3::class::basic::CompareOp;

# use pyo3::prelude::*;
#
# #[allow(dead_code)]
# #[pyclass]
# struct Number(i32);
#
#[pymethods]
impl Number {
    fn __richcmp__(&self, other: &Self, op: CompareOp) -> bool {
        op.matches(self.0.cmp(&other.0))
    }
}
```

It checks that the `std::cmp::Ordering` obtained from Rust's `Ord` matches
the given `CompareOp`.

Alternatively, you can implement just equality using `__eq__`:


```rust
# use pyo3::prelude::*;
#
# #[pyclass]
# struct Number(i32);
#
#[pymethods]
impl Number {
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

# fn main() -> PyResult<()> {
#     Python::attach(|py| {
#         let x = &Bound::new(py, Number(4))?;
#         let y = &Bound::new(py, Number(4))?;
#         assert!(x.eq(y)?);
#         assert!(!x.ne(y)?);
#         Ok(())
#     })
# }
```

To implement `__eq__` using the Rust [`PartialEq`] trait implementation, the `eq` option can be used.

```rust,no_run
# use pyo3::prelude::*;
#
# #[allow(dead_code)]
#[pyclass(eq)]
#[derive(PartialEq)]
struct Number(i32);
```

To implement `__lt__`, `__le__`, `__gt__`, & `__ge__` using the Rust `PartialOrd` trait implementation, the `ord` option can be used. *Note: Requires `eq`.*

```rust,no_run
# use pyo3::prelude::*;
#
# #[allow(dead_code)]
#[pyclass(eq, ord)]
#[derive(PartialEq, PartialOrd)]
struct Number(i32);
```

### Truthyness

We'll consider `Number` to be `True` if it is nonzero:

```rust,no_run
# use pyo3::prelude::*;
#
# #[allow(dead_code)]
# #[pyclass]
# struct Number(i32);
#
#[pymethods]
impl Number {
    fn __bool__(&self) -> bool {
        self.0 != 0
    }
}
```

### Final code

```rust,no_run
# fn main() {}
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use pyo3::prelude::*;
use pyo3::class::basic::CompareOp;
use pyo3::types::PyString;

#[pyclass]
struct Number(i32);

#[pymethods]
impl Number {
    #[new]
    fn new(value: i32) -> Self {
        Self(value)
    }

    fn __repr__(slf: &Bound<'_, Self>) -> PyResult<String> {
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
}

#[pymodule]
mod my_module {
    #[pymodule_export]
    use super::Number;
}
```

[`Hash`]: https://doc.rust-lang.org/std/hash/trait.Hash.html
[`Hasher`]: https://doc.rust-lang.org/std/hash/trait.Hasher.html
[`DefaultHasher`]: https://doc.rust-lang.org/std/collections/hash_map/struct.DefaultHasher.html
[SipHash]: https://en.wikipedia.org/wiki/SipHash
[`PartialEq`]: https://doc.rust-lang.org/stable/std/cmp/trait.PartialEq.html
